use std::sync::{Arc, RwLock, RwLockWriteGuard};

use cgmath::Vector3;

use crate::{gui::elements::{frame::Frame, screenui::ScreenUi, textlabel::TextLabel}, internal::raycaster::raycast_blocks, state::workspace::Workspace};

pub struct Debugger {
    seed: u32,
    camera_position_text: Arc<RwLock<TextLabel>>,
    camera_lookat_text: Arc<RwLock<TextLabel>>,
    target_block_text: Arc<RwLock<TextLabel>>,
    frame: Arc<RwLock<Frame>>
}

impl Debugger {
    pub fn new(seed: u32, screenui: &mut RwLockWriteGuard<ScreenUi>) -> Self {
        let frame = Frame::new("debugger-frame".to_owned());
        let camera_position_text = TextLabel::new("debugger-camera-position".to_owned(), "Camera Position: (0, 0, 0)".to_owned());
        let camera_lookat_text = TextLabel::new("debugger-camera-lookat".to_owned(), "Looking At: (0, 0, 0)".to_owned());
        let target_block_text = TextLabel::new("debugger-target-block".to_owned(), "Target Block: (0, 0, 0), [blockname]".to_owned());
        
        let mut frameread = frame.write().unwrap();
        frameread.add_child(camera_position_text.clone());
        frameread.add_child(camera_lookat_text.clone());
        frameread.add_child(target_block_text.clone());
        
        drop(frameread);

        screenui.add_child(frame.clone());
        
        Self {
            seed,
            camera_position_text,
            frame,
            camera_lookat_text,
            target_block_text
        }
    }

    pub fn update(&self, workspace_write: &parking_lot::lock_api::RwLockWriteGuard<parking_lot::RawRwLock, Workspace>) {

        let p = workspace_write.current_camera.position;

        let pos = Vector3::new(p.x, p.y, p.z);
        
        let res = raycast_blocks(pos, workspace_write.current_camera.look_vector(), 400.0, &workspace_write.chunk_manager, |_| false);
    
        match res {
            Some(hit) => {
                let abs = hit.hit.get_absolute_position();

                self.target_block_text.write().unwrap().set_text(format!("Target Block: ({}, {}, {}), [{}]", abs.x, abs.y, abs.z, hit.hit.get_name()));
            },
            None => {
                self.target_block_text.write().unwrap().set_text("Target Block: None".to_string());
            }
        }
    }
}