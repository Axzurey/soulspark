use crate::{gui::uistate::MouseButton, util::threadsignal::MonoThreadSignal};
use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{Button, Id};

pub enum UiAlignMode {
    Horizontal,
    Vertical
}


pub struct UiAlign {
    children: Vec<Box<dyn GuiObject>>,
    name: String,
    id: Id,
    position: GuiPosition,
    alignmode: UiAlignMode
}

impl UiAlign {
    pub fn new(name: String, text: String) -> Box<Self> {
        Box::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            alignmode: UiAlignMode::Vertical
        })
    }
}

impl GuiObject for UiAlign {
    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn GuiObject>> {
        &mut self.children
    }
    fn get_children(&self) -> &Vec<Box<dyn GuiObject>> {
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
                    self.children[i].render(ctx, ui);
                }
            }),
            UiAlignMode::Vertical => ui.vertical(|ui| {
                for i in 0..self.children.len() {
                    self.children[i].render(ctx, ui);
                }
            })
        };
    }
}