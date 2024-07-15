use cgmath::Vector2;
use eframe::egui::Id;

use crate::{gui::{guiobject::GuiObject, uistate::{GuiPosition, MouseButton}}, util::threadsignal::MonoThreadSignal};

pub struct Slider {
    children: Vec<Box<dyn GuiObject>>,
    name: String,
    id: Id,
    position: GuiPosition,
    enabled: bool,
    interactable: bool,
    pub on_move: MonoThreadSignal<MouseButton>,
    pub on_release: MonoThreadSignal<()>,
    pub min: f32,
    pub max: f32,
    pub value: f32
}

impl Slider {
    pub fn new(name: String) -> Box<Self> {
        Box::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            position: GuiPosition::Position(Vector2::new(0., 0.)),
            enabled: true,
            interactable: true,
            on_move: MonoThreadSignal::new(),
            on_release: MonoThreadSignal::new(),
            max: 10.0,
            min: 0.0,
            value: 2.5
        })
    }
}

impl GuiObject for Slider {
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
        let r = self.min..=self.max;
        let sliderraw = eframe::egui::Slider::new(&mut self.value, r);
        
        let slider = ui.add_sized([200., 50.], sliderraw);
    }
    
    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn GuiObject>> {
        &mut self.children
    }
}