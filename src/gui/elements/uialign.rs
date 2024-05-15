use crate::{gui::uistate::MouseButton, util::threadsignal::MonoThreadSignal};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{Button, Id};

pub enum UiAlignMode {
    Horizontal,
    Vertical
}


pub struct UiAlign {
    children: Vec<Arc<RwLock<dyn GuiObject>>>,
    name: String,
    id: Id,
    position: GuiPosition,
    alignmode: UiAlignMode
}

impl UiAlign {
    pub fn new(name: String, text: String) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            alignmode: UiAlignMode::Vertical
        }))
    }
}

impl GuiObject for RwLockWriteGuard<'_, UiAlign> {
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
        
        ui.horizontal(add_contents)
    }
}

impl GuiObject for UiAlign {
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