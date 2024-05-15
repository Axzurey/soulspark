use crate::util::threadsignal::MonoThreadSignal;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{Label, Id};
use pollster::FutureExt;

pub struct TextLabel {
    children: Vec<Arc<RwLock<dyn GuiObject>>>,
    name: String,
    id: Id,
    position: GuiPosition,
    enabled: bool,
    interactable: bool,
    pub on_hover_enter: MonoThreadSignal<()>,
    text: String,

    hovered: bool,
}

impl TextLabel {
    pub fn new(name: String, text: String) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            enabled: true,
            interactable: true,
            on_hover_enter: MonoThreadSignal::new(),
            text,
            hovered: false
        }))
    }
}

impl GuiObject for RwLockWriteGuard<'_, TextLabel> {
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
        let labelraw = Label::new(&self.text);
        
        let label = ui.add_sized([200., 50.], labelraw);

        if label.hovered() && !self.hovered {
            self.hovered = true;
            self.on_hover_enter.dispatch(()).block_on();
        }
    }
}

impl GuiObject for TextLabel {
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
        let labelraw = Label::new(&self.text);
        
        let label = ui.add_sized([200., 50.], labelraw);

        if label.hovered() && !self.hovered {
            self.hovered = true;
            self.on_hover_enter.dispatch(()).block_on();
        }
        else {
            self.hovered = false;
        }
    }
}