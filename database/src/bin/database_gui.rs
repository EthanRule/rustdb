use database::ui::DatabaseApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                // You can add an icon here if you have one
                eframe::icon_data::from_png_bytes(&[])
                    .unwrap_or_default()
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Database Engine Explorer",
        options,
        Box::new(|_cc| {
            // Customize egui style
            let mut style = egui::Style::default();
            style.visuals.dark_mode = true; // You can make this configurable
            _cc.egui_ctx.set_style(style);
            
            Box::new(DatabaseApp::new())
        }),
    )
}