use eframe::egui::Style;
use eframe::egui::style::WidgetVisuals;

pub fn all_visuals(style: &mut Style, f: impl Fn(&mut WidgetVisuals)) {
    f(&mut style.visuals.widgets.active);
    f(&mut style.visuals.widgets.hovered);
    f(&mut style.visuals.widgets.inactive);
    f(&mut style.visuals.widgets.noninteractive);
    f(&mut style.visuals.widgets.open);
}
