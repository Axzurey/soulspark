use cgmath::Vector2;
use eframe::egui::Align2;

#[derive(Clone, Copy, PartialEq)]
pub enum GuiPosition {
    Locked(Vector2<f32>),
    Position(Vector2<f32>),
    Anchor((Align2, Vector2<f32>))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16)
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Back => MouseButton::Back,
            winit::event::MouseButton::Forward => MouseButton::Forward,
            winit::event::MouseButton::Other(i) => MouseButton::Other(i),
        }
    }
}