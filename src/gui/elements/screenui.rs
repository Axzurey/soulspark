use std::any::Any;

use crate::util::helpers::get_typed;

use super::super::{guiobject::GuiObject};
use eframe::egui::{self, Id};

pub struct ScreenUi {
    children: Vec<Box<dyn GuiObject>>,
    name: String,
    id: Id,
    enabled: bool,
    interactable: bool
}

impl ScreenUi {
    pub fn new(name: String) -> Box<Self> {
        Box::new(Self {
            children: Vec::new(),
            name: name.clone(),
            id: Id::new(name),
            enabled: true,
            interactable: true
        })
    }
    pub fn find_first_child_mut(&mut self, name: String) -> Option<&mut Box<dyn GuiObject>> {
        for child in self.get_children_mut() {
            if child.get_name() == name {
                return Some(child)
            }
        }
        None
    }
    //attempts to find child based on "path"
    //example: search_for_mut("someframe/sometextbutton")
    pub fn search_for_mut<T: 'static>(&mut self, path: String) -> Option<&mut T> {
        let mut slices = path.split("/");

        match slices.next() {
            Some(first) => {
                let somefirst = self.find_first_child_mut(first.to_string());
                slices.fold(somefirst, |acc, name| {
                    if acc.is_some() {
                        return acc.unwrap().find_first_child_mut(name.to_string())
                    }
                    else {
                        return acc
                    }
                }).map(|v| {
                    let value_any = v;
                    get_typed::<T>(value_any)
                }).flatten()
            },
            None => None
        }
    }
    pub fn get_children_mut(&mut self) -> &mut Vec<Box<dyn GuiObject>> {
        &mut self.children
    }
    pub fn add_child(&mut self, child: Box<dyn GuiObject>) {
        self.children.push(child);
    }
    pub fn get_children(&self) -> &Vec<Box<dyn GuiObject>> {
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
                self.children[i].render(ctx, ui);
            }
        });
    }
}
