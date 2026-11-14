mod api;
mod cli;
mod installer;
mod wizard;

use wizard::WizardApp;

fn main() -> eframe::Result<()> {
    dotenvy::dotenv().ok();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "NetBird Setup Wizard",
        options,
        Box::new(|_cc| Ok(Box::new(WizardApp::new()))),
    )
}
