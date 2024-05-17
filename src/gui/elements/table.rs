use std::{collections::HashMap, sync::{Arc, RwLock, RwLockWriteGuard}};

use eframe::egui::{self, Id, ScrollArea, TextBuffer, TextEdit};
use egui_extras::{Column, TableBuilder};
use rand::{distributions::Alphanumeric, Rng};

use crate::gui::guiobject::GuiObject;

pub struct Table<V: eframe::egui::TextBuffer + Clone> {
    name: String,
    id: Id,
    children: Vec<Arc<RwLock<dyn GuiObject>>>,
    n_rows: u32,
    keys: Vec<String>,
    values: Vec<V>,
    default_value: V
}

impl<V: eframe::egui::TextBuffer + Clone> Table<V> {
    pub fn new(name: String, default_value: V) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            name: name.clone(),
            id: Id::new(name),
            children: Vec::new(),
            n_rows: 0,
            keys: Vec::new(),
            values: Vec::new(),
            default_value
        }))
    }
}

impl<V: eframe::egui::TextBuffer + Clone> GuiObject for RwLockWriteGuard<'_, Table<V>> {
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

impl<V: eframe::egui::TextBuffer + Clone> GuiObject for Table<V> {
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
                body.row(15.0, |mut row| {
                    row.col(|ui| {
                        let t = ui.text_edit_singleline(&mut self.keys[i]);
                    });
                    row.col(|ui| {
                        let t = ui.text_edit_singleline(&mut self.values[i]);
                    });
                });
            }
            body.row(15.0, |mut row| {
                row.col(|ui| {
                    let btn = ui.button("Add Row");
                    if btn.clicked() {
                        let s: String = rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(7)
                            .map(char::from)
                            .collect();
                        self.keys.push(s);
                        self.values.push(self.default_value.clone());
                    }
                });
            });
        });
    }
}