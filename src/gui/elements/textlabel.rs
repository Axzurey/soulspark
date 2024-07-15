use crate::util::threadsignal::MonoThreadSignal;
use super::super::{guiobject::GuiObject, uistate::GuiPosition};
use cgmath::Vector2;
use eframe::egui::{Color32, Id, Label, RichText};
use pollster::FutureExt;

pub struct TextLabel {
    children: Vec<Box<dyn GuiObject>>,
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

    pub fn new(name: String, text: String) -> Box<Self> {
        Box::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            enabled: true,
            interactable: true,
            on_hover_enter: MonoThreadSignal::new(),
            text,
            hovered: false
        })
    }
    pub fn set_text(&mut self, t: String) {
        self.text = t;
    }
    pub fn get_text(&mut self) -> String {
        self.text.clone()
    }
}

impl GuiObject for TextLabel {
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
        let labelraw = Label::new(RichText::new(self.text.clone()).heading().color(Color32::from_rgb(255, 255, 255)));
        
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