use std::sync::{Arc, RwLock, RwLockWriteGuard};
use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{self, Id};

pub struct ScreenUi {
    children: Vec<Arc<RwLock<dyn GuiObject>>>,
    name: String,
    id: Id,
    enabled: bool,
    interactable: bool
}

impl ScreenUi {
    pub fn new(name: String) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            enabled: true,
            interactable: true
        }))
    }
    pub fn add_child(&mut self, child: Arc<RwLock<dyn GuiObject>>) {
        self.children.push(child);
    }
    pub fn get_children(&self) -> &Vec<Arc<RwLock<dyn GuiObject>>> {
        &self.children
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn render(&mut self, ctx: &eframe::egui::Context) {

        let area = egui::Area::new(self.id)
        .enabled(self.enabled)
        .interactable(self.interactable);

        area.show(ctx, |ui| {
            for i in 0..self.children.len() {
                self.children[i].write().unwrap().render(ctx, ui);
            }
        });
    }
}
