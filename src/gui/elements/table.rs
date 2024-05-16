use std::{collections::HashMap, sync::{Arc, RwLock, RwLockWriteGuard}};

use eframe::egui::{self, Id, ScrollArea, TextBuffer, TextEdit};
use egui_extras::{Column, TableBuilder};
use splines::Spline;

use crate::gui::guiobject::GuiObject;

pub struct Table<K: std::cmp::Eq + std::hash::Hash + Into<String>, V> {
    adornee: HashMap<K, V>,
    name: String,
    id: Id,
    children: Vec<Arc<RwLock<dyn GuiObject>>>,
    n_rows: u32,
    keys: Vec<K>
}

impl<K: std::cmp::Eq + std::hash::Hash + Into<String>, V> Table<K, V> {
    pub fn new(name: String, adornee: HashMap<K, V>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            name: name.clone(),
            id: Id::new(name),
            adornee,
            children: Vec::new(),
            n_rows: 0,
            keys: Vec::new()
        }))
    }
}

impl<K: std::cmp::Eq + std::hash::Hash + Into<String>, V> GuiObject for RwLockWriteGuard<'_, Table<K, V>> {
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
        
    }
}

impl<K: std::cmp::Eq + std::hash::Hash + Into<String>, V> GuiObject for Table<K, V> {
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
        let table = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto());

        table.header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Key");
            });
            header.col(|ui| {
                ui.strong("Value");
            });
        })
        .body(|mut body| {
            for i in 0..self.keys.len() {
                let key = &self.keys[i];
                let value = &self.adornee[key];

                body.row(15.0, |mut row| {
                    row.col(|ui| {
                        let text_buffer = 
                        let t = TextEdit::singleline(key.into());
                    });
                });
            }
        });
    }
}