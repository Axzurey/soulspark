use std::sync::Arc;

use crate::gen::object::Object;

pub struct RenderStorage {
    objects: Vec<Arc<Object>>
}

impl RenderStorage {
    pub fn new() -> Self {
        Self {
            objects: Vec::new()
        }
    }
    pub fn add_object(&mut self, object: Arc<Object>) {
        self.objects.push(object);
    }
    pub fn remove_object<C>(&mut self, callback: C) -> Option<Arc<Object>> where C: Fn(&Arc<Object>) -> bool {
        let mut i = 0;
        let mut found: Option<Arc<Object>> = None;
        for object in &self.objects {
            if callback(object) {
                found = Some(object.clone());
                break;
            }
            i += 1;
        }
        match found {
            Some(f) => {
                self.objects.remove(i);
                Some(f)
            },
            None => None
        }
    }
    pub fn get_objects(&self) -> &Vec<Arc<Object>> {
        &self.objects
    }
    pub fn get_object<C>(&self, callback: C) -> Option<Arc<Object>> where C: Fn(&Arc<Object>) -> bool {
        for object in &self.objects {
            if callback(object) {
                return Some(object.clone());
            }
        }
        None
    }
}