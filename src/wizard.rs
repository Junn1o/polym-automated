use std::sync::mpsc::{channel, Receiver};
use std::thread;

use eframe::egui;

use crate::api::{self, InviteRequest};
use crate::cli;
use crate::installer;

const INSTALLER_URL: &str = "https://pkgs.netbird.io/windows/x64";

fn installer_path() -> std::path::PathBuf {
    std::env::temp_dir().join("netbird_installer.exe")
}

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
    auto_groups: Vec<String>,
    email_input: String,
    name_input: String,
    role_input: String,
}

impl WizardApp {
    pub fn new() -> Self {
        let domain = std::env::var("NETBIRD_DOMAIN").unwrap_or_default();
        let bearer_token = std::env::var("API_BEARER_TOKEN").unwrap_or_default();
        let auto_groups = std::env::var("AUTO_GROUP_ID")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            step: Step::ChooseAction,
            invite_url: None,
            rx: None,
            domain,
            bearer_token,
            auto_groups,
            email_input: String::new(),
            name_input: String::new(),
            role_input: "user".to_string(),
        }
    }

    fn start_install(&mut self) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.step = Step::Installing;

        let domain = self.domain.clone();

        thread::spawn(move || {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                let path = installer_path();
                let path_str = path.to_str().ok_or("Đường dẫn file cài đặt không hợp lệ")?;
                installer::download_file(INSTALLER_URL, path_str)?;
                let ok = installer::run_installer(path_str)?;
                if ok {
                    installer::delete_file(path_str)?;
                }
                if !domain.is_empty() {
                    let management_url = format!("https://{}:443", domain);
                    cli::netbird_up_silent(&management_url);
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
        let name = self.name_input.clone();
        let role = self.role_input.clone();
        let auto_groups = self.auto_groups.clone();

        thread::spawn(move || {
            let request = InviteRequest {
                email,
                name,
                role,
                auto_groups,
                expires_in: 259200,
            };

            let invite_api_url = format!("https://{}/api/users/invites", domain);
            let result = api::create_invite(&invite_api_url, &bearer_token, &request);

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
                    let _ = webbrowser::open(&url);
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
                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.email_input);
                });
                ui.horizontal(|ui| {
                    ui.label("Tên:");
                    ui.text_edit_singleline(&mut self.name_input);
                });
                ui.horizontal(|ui| {
                    ui.label("Role:");
                    egui::ComboBox::from_id_salt("role_select")
                        .selected_text(&self.role_input)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.role_input, "user".to_string(), "user");
                            ui.selectable_value(&mut self.role_input, "admin".to_string(), "admin");
                        });
                });
                let can_submit = !self.email_input.is_empty() && !self.name_input.is_empty();
                if ui.add_enabled(can_submit, egui::Button::new("Tạo invite")).clicked() {
                    self.start_create_invite();
                }
                if ui.button("Đã có account, kết nối luôn").clicked() {
                    if !self.domain.is_empty() {
                        let _ = webbrowser::open(&format!("https://{}", self.domain));
                    }
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
                    cli::open_netbird_ui();
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
