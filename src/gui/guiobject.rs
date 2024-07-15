use eframe::egui::{Context, Ui};

pub trait GuiObject {
    fn get_children(&self) -> &Vec<Box<dyn GuiObject>>;
    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn GuiObject>>;

    fn find_first_child_mut(&mut self, name: String) -> Option<&mut Box<dyn GuiObject>> {
        for child in self.get_children_mut() {
            if child.get_name() == name {
                return Some(child)
            }
        }
        None
    }

    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: String);

    fn render(&mut self, ctx: &Context, ui: &mut Ui);
}