use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{self, Id};

pub struct Frame {
    children: Vec<Box<dyn GuiObject>>,
    name: String,
    id: Id,
    position: GuiPosition,
    enabled: bool,
    interactable: bool
}

impl Frame {
    pub fn new(name: String) -> Box<Self> {
        Box::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            enabled: true,
            interactable: true
        })
    }
    pub fn add_child(&mut self, child: Box<dyn GuiObject>) {
        self.children.push(child);
    }
}

impl GuiObject for Frame {
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
                self.children[i].render(ctx, ui);
            }
        });
    }
}