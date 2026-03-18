use database::ui::DatabaseApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 860.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "rustdb",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.window_rounding = egui::Rounding::same(6.0);
            visuals.panel_fill = egui::Color32::from_rgb(18, 20, 26);
            visuals.override_text_color = Some(egui::Color32::from_rgb(210, 215, 225));
            visuals.selection.bg_fill = egui::Color32::from_rgb(130, 55, 10);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 33, 42);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 44, 56);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 55, 70);
            visuals.extreme_bg_color = egui::Color32::from_rgb(14, 15, 20);
            visuals.code_bg_color = egui::Color32::from_rgb(22, 24, 32);
            cc.egui_ctx.set_visuals(visuals);

            let mut style = (*cc.egui_ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
            style.spacing.button_padding = egui::vec2(10.0, 5.0);
            cc.egui_ctx.set_style(style);

            Box::new(DatabaseApp::new())
        }),
    )
}
