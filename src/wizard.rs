use std::sync::mpsc::{channel, Receiver};
use std::thread;

use eframe::egui;

use crate::api::{self, InviteRequest};
use crate::cli;
use crate::installer;

const INSTALLER_URL: &str = "https://pkgs.netbird.io/windows/x64";
const INSTALLER_PATH: &str = "netbird_installer.exe";
const INVITE_API_URL: &str = "https://api.netbird.io/api/users/invites";

#[derive(PartialEq, Clone)]
enum Step {
    ChooseAction,
    Installing,
    ServerCheck,
    CreatingInvite,
    WaitingForPassword,
    ConfiguringClient,
    Connecting,
    Done,
    Error(String),
}

enum WorkerMsg {
    InstallDone,
    InstallFailed(String),
    InviteDone(String),
    InviteFailed(String),
    ConnectDone,
    ConnectFailed(String),
}

pub struct WizardApp {
    step: Step,
    invite_url: Option<String>,
    rx: Option<Receiver<WorkerMsg>>,
    domain: String,
    bearer_token: String,
    email_input: String,
}

impl WizardApp {
    pub fn new() -> Self {
        let domain = std::env::var("NETBIRD_DOMAIN").unwrap_or_default();
        let bearer_token = std::env::var("API_BEARER_TOKEN").unwrap_or_default();

        Self {
            step: Step::ChooseAction,
            invite_url: None,
            rx: None,
            domain,
            bearer_token,
            email_input: String::new(),
        }
    }

    fn start_install(&mut self) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.step = Step::Installing;

        thread::spawn(move || {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                installer::download_file(INSTALLER_URL, INSTALLER_PATH)?;
                let ok = installer::run_installer(INSTALLER_PATH)?;
                if ok {
                    installer::delete_file(INSTALLER_PATH)?;
                }
                Ok(())
            })();

            match result {
                Ok(()) => tx.send(WorkerMsg::InstallDone).ok(),
                Err(e) => tx.send(WorkerMsg::InstallFailed(e.to_string())).ok(),
            };
        });
    }

    fn start_create_invite(&mut self) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.step = Step::CreatingInvite;

        let bearer_token = self.bearer_token.clone();
        let domain = self.domain.clone();
        let email = self.email_input.clone();

        thread::spawn(move || {
            let request = InviteRequest {
                email,
                name: "New User".to_string(),
                role: "user".to_string(),
                auto_groups: vec![],
                expires_in: 259200,
            };

            let result = api::create_invite(INVITE_API_URL, &bearer_token, &request);

            match result {
                Ok(invite) => {
                    let url = api::build_invite_url(&domain, &invite.invite_token);
                    tx.send(WorkerMsg::InviteDone(url)).ok()
                }
                Err(e) => tx.send(WorkerMsg::InviteFailed(e.to_string())).ok(),
            };
        });
    }

    fn start_connect(&mut self) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.step = Step::Connecting;

        let domain = self.domain.clone();

        thread::spawn(move || {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let management_url = format!("https://{}:443", domain);
                cli::netbird_up(&management_url)?;
                let status = cli::netbird_status()?;
                if !cli::is_connected(&status) {
                    Err(format!("NetBird not connected. Status output: {}", status))?;
                }
                Ok(())
            })();

            match result {
                Ok(()) => tx.send(WorkerMsg::ConnectDone).ok(),
                Err(e) => tx.send(WorkerMsg::ConnectFailed(e.to_string())).ok(),
            };
        });
    }

    fn poll_worker(&mut self) {
        let Some(rx) = &self.rx else { return };

        if let Ok(msg) = rx.try_recv() {
            match msg {
                WorkerMsg::InstallDone => self.step = Step::ServerCheck,
                WorkerMsg::InstallFailed(e) => self.step = Step::Error(e),
                WorkerMsg::InviteDone(url) => {
                    self.invite_url = Some(url);
                    self.step = Step::WaitingForPassword;
                }
                WorkerMsg::InviteFailed(e) => self.step = Step::Error(e),
                WorkerMsg::ConnectDone => self.step = Step::Done,
                WorkerMsg::ConnectFailed(e) => self.step = Step::Error(e),
            }
            self.rx = None;
        }
    }
}

impl eframe::App for WizardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();
        if self.rx.is_some() {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.step.clone() {
            Step::ChooseAction => {
                ui.heading("NetBird Setup Wizard");
                ui.label("Bạn muốn làm gì?");
                if ui.button("Cài đặt NetBird").clicked() {
                    self.start_install();
                }
                if ui.button("Set server").clicked() {
                    self.step = Step::ServerCheck;
                }
            }
            Step::Installing => {
                ui.heading("Đang cài đặt...");
                ui.spinner();
            }
            Step::ServerCheck => {
                ui.heading("Cấu hình server");
                ui.label("Bạn đã có account chưa?");
                if ui.button("Chưa có, tạo account mới").clicked() {
                    self.step = Step::CreatingInvite;
                }
                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.email_input);
                });
                if ui.button("Tạo invite").clicked() && !self.email_input.is_empty() {
                    self.start_create_invite();
                }
                if ui.button("Đã có account, kết nối luôn").clicked() {
                    self.step = Step::ConfiguringClient;
                }
            }
            Step::CreatingInvite => {
                ui.heading("Đang tạo invite...");
                ui.spinner();
            }
            Step::WaitingForPassword => {
                ui.heading("Tạo mật khẩu");
                ui.label("Mở trình duyệt để đặt mật khẩu cho account mới:");
                if let Some(url) = self.invite_url.clone() {
                    ui.label(&url);
                    if ui.button("Mở trình duyệt").clicked() {
                        let _ = webbrowser::open(&url);
                    }
                }
                if ui.button("Đã đặt mật khẩu xong, tiếp tục").clicked() {
                    self.step = Step::ConfiguringClient;
                }
            }
            Step::ConfiguringClient => {
                ui.heading("Cấu hình NetBird client");
                ui.label(format!("Self-hosted domain: {}", self.domain));
                ui.label("NetBird sẽ tự mở trình duyệt để xác thực khi kết nối.");
                if ui.button("Kết nối").clicked() {
                    self.start_connect();
                }
            }
            Step::Connecting => {
                ui.heading("Đang kết nối NetBird...");
                ui.spinner();
            }
            Step::Done => {
                ui.heading("Hoàn tất!");
                ui.label("NetBird đã kết nối thành công.");
                if ui.button("Đóng").clicked() {
                    std::process::exit(0);
                }
            }
            Step::Error(msg) => {
                ui.heading("Lỗi");
                ui.colored_label(egui::Color32::RED, &msg);
                if ui.button("Thử lại").clicked() {
                    self.step = Step::ChooseAction;
                }
            }
        });
    }
}
