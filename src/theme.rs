//! 1980s CRT terminal theme: phosphor green on black, hard edges, scanlines.

use eframe::egui::{self, Color32, Stroke};

pub const BG: Color32 = Color32::from_rgb(4, 8, 6);
pub const PANEL: Color32 = Color32::from_rgb(8, 18, 12);
pub const GREEN: Color32 = Color32::from_rgb(51, 255, 102);
pub const GREEN_DIM: Color32 = Color32::from_rgb(18, 110, 50);
pub const AMBER: Color32 = Color32::from_rgb(255, 176, 0);
pub const RED: Color32 = Color32::from_rgb(255, 56, 56);

/// Apply the retro visual style to the context.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(GREEN);
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(1, 4, 2);
    visuals.faint_bg_color = PANEL;
    visuals.window_stroke = Stroke::new(2.0_f32, GREEN_DIM);
    visuals.selection.bg_fill = GREEN_DIM;
    visuals.selection.stroke = Stroke::new(1.0_f32, GREEN);

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = PANEL;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, GREEN_DIM);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, GREEN);
    widgets.inactive.bg_fill = Color32::from_rgb(12, 30, 20);
    widgets.inactive.bg_stroke = Stroke::new(1.0_f32, GREEN_DIM);
    widgets.inactive.fg_stroke = Stroke::new(1.0_f32, GREEN);
    widgets.hovered.bg_fill = Color32::from_rgb(18, 54, 32);
    widgets.hovered.bg_stroke = Stroke::new(1.0_f32, GREEN);
    widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    widgets.active.bg_fill = Color32::from_rgb(26, 84, 48);
    widgets.active.bg_stroke = Stroke::new(1.0_f32, GREEN);
    widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::monospace(20.0)),
        (egui::TextStyle::Body, egui::FontId::monospace(14.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(14.0)),
        (egui::TextStyle::Button, egui::FontId::monospace(14.0)),
        (egui::TextStyle::Small, egui::FontId::monospace(11.0)),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(6.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style(style);
}

/// Draw subtle CRT scanlines over the whole window.
pub fn scanlines(ctx: &egui::Context) {
    let screen = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("scanlines"),
    ));
    let color = Color32::from_rgba_unmultiplied(0, 0, 0, 30);
    let stroke = Stroke::new(1.0_f32, color);
    let mut y = screen.top();
    while y < screen.bottom() {
        painter.hline(screen.x_range(), y, stroke);
        y += 3.0;
    }
}
