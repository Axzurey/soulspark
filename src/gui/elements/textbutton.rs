use crate::{gui::uistate::MouseButton, util::threadsignal::MonoThreadSignal};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{Button, Id};
use pollster::FutureExt;

pub struct TextButton {
    children: Vec<Arc<RwLock<dyn GuiObject>>>,
    name: String,
    id: Id,
    position: GuiPosition,
    enabled: bool,
    interactable: bool,
    pub on_click: MonoThreadSignal<MouseButton>,
    pub on_hover_enter: MonoThreadSignal<()>,
    text: String,

    hovered: bool,
    left_mouse_down: bool,
    right_mouse_down: bool
}

impl TextButton {
    pub fn new(name: String, text: String) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            enabled: true,
            interactable: true,
            on_click: MonoThreadSignal::new(),
            on_hover_enter: MonoThreadSignal::new(),
            text,
            hovered: false,
            left_mouse_down: false,
            right_mouse_down: false
        }))
    }
}

impl GuiObject for RwLockWriteGuard<'_, TextButton> {
    fn get_children(&self) -> &Vec<Arc<RwLock<dyn GuiObject>>> {
        &self.children
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut eframe::egui::Ui) {
        let buttonraw = Button::new(&self.text);
        
        let button = ui.add_sized([200., 50.], buttonraw);
        
        if button.clicked() && !self.left_mouse_down {
            self.left_mouse_down = true;
            self.on_click.dispatch(MouseButton::Left).block_on();
        }
        if button.secondary_clicked() && !self.right_mouse_down {
            self.right_mouse_down = true;
            self.on_click.dispatch(MouseButton::Right).block_on();
        }

        if button.hovered() && !self.hovered {
            self.hovered = true;
            self.on_hover_enter.dispatch(()).block_on();
        }
    }
}

impl GuiObject for TextButton {
    fn get_children(&self) -> &Vec<Arc<RwLock<dyn GuiObject>>> {
        &self.children
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut eframe::egui::Ui) {
        let buttonraw = Button::new(&self.text);
        
        let button = ui.add_sized([200., 50.], buttonraw);
        if button.clicked() && !self.left_mouse_down {
            self.left_mouse_down = true;
            self.on_click.dispatch(MouseButton::Left).block_on();
        }
        else {
            self.left_mouse_down = false;
        }
        if button.secondary_clicked() && !self.right_mouse_down {
            self.right_mouse_down = true;
            self.on_click.dispatch(MouseButton::Right).block_on();
        }
        else {
            self.right_mouse_down = false;
        }
        if button.hovered() && !self.hovered {
            self.hovered = true;
            self.on_hover_enter.dispatch(()).block_on();
        }
        else {
            self.hovered = false;
        }
    }
}