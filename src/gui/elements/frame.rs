use std::sync::{Arc, RwLock};
use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{self, Id};

pub struct Frame {
    children: Vec<Arc<RwLock<dyn GuiObject>>>,
    name: String,
    id: Id,
    position: GuiPosition,
    enabled: bool,
    interactable: bool
}

impl Frame {
    pub fn new(name: String) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            enabled: true,
            interactable: true
        }))
    }
    pub fn add_child(&mut self, child: Arc<RwLock<dyn GuiObject>>) {
        self.children.push(child);
    }
}

impl GuiObject for Frame {
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
        let area = egui::Area::new(self.id)
        .enabled(self.enabled)
        .interactable(self.interactable);

        match self.position {
            GuiPosition::Anchor(s) => {
                let _ = area.anchor(s.0, (s.1.x, s.1.y));
            },
            GuiPosition::Locked(s) => {
                let _ = area.fixed_pos((s.x, s.y));
            },
            GuiPosition::Position(s) => {
                let _ = area.current_pos((s.x, s.y));
            }
        }

        area.show(ctx, |ui| {
            for i in 0..self.children.len() {
                self.children[i].write().unwrap().render(ctx, ui);
            }
        });
    }
}