use std::sync::{Arc, RwLock};

use eframe::egui::{Context, Response, Ui};

pub trait GuiObject {
    fn get_children(&self) -> &Vec<Arc<RwLock<dyn GuiObject>>>;

    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: String);

    fn render(&mut self, ctx: &Context, ui: &mut Ui);
}