mod api;
mod cli;
mod installer;
mod wizard;

use wizard::WizardApp;

fn load_vietnamese_font(ctx: &eframe::egui::Context) {
    let candidates = [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    let Some(bytes) = candidates.iter().find_map(|p| std::fs::read(p).ok()) else {
        return;
    };

    let mut fonts = eframe::egui::FontDefinitions::default();
    fonts.font_data.insert(
        "system_vi".to_owned(),
        eframe::egui::FontData::from_owned(bytes),
    );
    fonts
        .families
        .entry(eframe::egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "system_vi".to_owned());
    fonts
        .families
        .entry(eframe::egui::FontFamily::Monospace)
        .or_default()
        .push("system_vi".to_owned());
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    dotenvy::dotenv().ok();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "NetBird Setup Wizard",
        options,
        Box::new(|cc| {
            load_vietnamese_font(&cc.egui_ctx);
            Ok(Box::new(WizardApp::new()))
        }),
    )
}
