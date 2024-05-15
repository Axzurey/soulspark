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

        let layout = match self.alignmode {
            UiAlignMode::Horizontal => ui.horizontal(|ui| {
                for i in 0..self.children.len() {
                    self.children[i].write().unwrap().render(ctx, ui);
                }
            }),
            UiAlignMode::Vertical => ui.vertical(|ui| {
                for i in 0..self.children.len() {
                    self.children[i].write().unwrap().render(ctx, ui);
                }
            })
        };
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
        let layout = match self.alignmode {
            UiAlignMode::Horizontal => ui.horizontal(|ui| {
                for i in 0..self.children.len() {
                    self.children[i].write().unwrap().render(ctx, ui);
                }
            }),
            UiAlignMode::Vertical => ui.vertical(|ui| {
                for i in 0..self.children.len() {
                    self.children[i].write().unwrap().render(ctx, ui);
                }
            })
        };
    }
}