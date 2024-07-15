use std::sync::{Arc, RwLock, RwLockWriteGuard};

use cgmath::Vector3;

use crate::{gui::elements::{frame::Frame, screenui::ScreenUi, textlabel::TextLabel}, internal::raycaster::raycast_blocks, state::workspace::Workspace, vox::chunk_manager::ChunkManager};

use super::helpers::get_typed;

pub struct Debugger {
    seed: u32,
    //camera_position_text: Box<TextLabel>,
    //camera_lookat_text: Box<TextLabel>,
    //target_block_text: Box<TextLabel>,
    //frame: Box<Frame>
}

impl Debugger {
    pub fn new(seed: u32, screenui: &mut ScreenUi) -> Self {
        let mut frame = Frame::new("debugger-frame".to_owned());
        let camera_position_text = TextLabel::new("debugger-camera-position".to_owned(), "Camera Position: (0, 0, 0)".to_owned());
        let camera_lookat_text = TextLabel::new("debugger-camera-lookat".to_owned(), "Looking At: (0, 0, 0)".to_owned());
        let target_block_text = TextLabel::new("debugger-target-block".to_owned(), "Target Block: (0, 0, 0), [blockname]".to_owned());
        
        frame.add_child(camera_position_text);
        frame.add_child(camera_lookat_text);
        frame.add_child(target_block_text);

        screenui.add_child(frame);
        
        Self {
            seed
            //camera_position_text,
            //frame,
            //camera_lookat_text,
            //target_block_text
        }
    }

    pub fn update(&mut self, workspace_write: &parking_lot::lock_api::RwLockWriteGuard<parking_lot::RawRwLock, Workspace>, screengui: &mut ScreenUi) {

        let p = workspace_write.current_camera.position;

        let pos = Vector3::new(p.x, p.y, p.z);
        
        let res = raycast_blocks(pos, workspace_write.current_camera.look_vector(), 400.0, &workspace_write.chunk_manager, |_| false);
        
        let target_block_text = screengui.search_for_mut::<Box<TextLabel>>("debugger-frame/debugger-target-block".to_owned()).unwrap();

        match res {
            Some(hit) => {
                let abs = hit.hit.get_absolute_position();

                target_block_text.set_text(format!("Target Block: ({}, {}, {}), [{}], normal light: {}", abs.x, abs.y, abs.z, hit.hit.get_name(), ChunkManager::get_sunlight_intensity_at(abs.x + hit.normal.x, (abs.y + hit.normal.y) as u32, abs.z + hit.normal.z, &workspace_write.chunk_manager.chunks)));
            },
            None => {
                target_block_text.set_text("Target Block: None".to_string());
            }
        }
    }
}