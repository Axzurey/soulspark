use cgmath::Vector2;
use eframe::egui::Align2;

pub enum GuiPosition {
    Locked(Vector2<f32>),
    Position(Vector2<f32>),
    Anchor((Align2, Vector2<f32>))
}

pub enum MouseButton {
    Left,
    Right,
    Middle
}