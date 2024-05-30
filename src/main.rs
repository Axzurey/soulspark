use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};

use blocks::stoneblock::StoneBlock;
use cgmath::{Point3, Vector3};
use engine::surfacevertex::SurfaceVertex;
use gen::primitive::PrimitiveBuilder;
use gui::elements::slider::Slider;
use gui::elements::table::Table;
use gui::elements::textbutton::TextButton;
use gui::elements::textlabel::TextLabel;
use gui::uistate::MouseButton;
use internal::raycaster::raycast_blocks;
use internal::window::GameWindow;
use pollster::FutureExt;
use state::workspace::Workspace;
use stopwatch::Stopwatch;
use util::inputservice::{InputService, MouseLockState};
use vox::chunk::{xz_to_index, ChunkGridType};
use vox::chunk_manager::mesh_slice_arrayed;
use vox::chunkactionqueue::ChunkAction;
use vox::structure_loader::load_structures;
use winit::event::{DeviceEvent, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

mod internal;
mod state;
mod engine;
mod vox;
mod gen;
mod gui;
mod blocks;
mod util;

async fn dosomth(x: &u32) {
    
}

#[tokio::main()]
async fn main() {

    let a: Arc<Vec<u32>> = Arc::new(Vec::new());

    //find way to edit arc

    let event_loop = EventLoop::new().unwrap();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    load_structures();
    
    let mut gamewindow = GameWindow::new(window.clone()).block_on();

    let mut workspace_arc = Arc::new(RwLock::new(Workspace::new(
        &gamewindow.device, &gamewindow.camera_bindgroup_layout, 
        gamewindow.window_size.width, gamewindow.window_size.height,
        window.clone()
    )));

    let mut workspace = workspace_arc.write().unwrap();

    workspace.chunk_manager.generate_chunks(&gamewindow.device);
    workspace.chunk_manager.generate_chunk_illumination();
    workspace.chunk_manager.mesh_chunks(&gamewindow.device);

    let text_label = TextLabel::new("h".to_owned(), "hellotext".to_owned());

    {
        let wa = workspace_arc.clone();
        let cloned_label = text_label.clone();

        workspace.input_service.on_mouse_click.connect(move |(btn, _)| {

            let lock = &mut wa.write().unwrap();

            let p = lock.current_camera.position;

            let pos = Vector3::new(p.x, p.y, p.z);
            
            let res = raycast_blocks(pos, lock.current_camera.look_vector(), 400.0, &lock.chunk_manager, |_| false);
        
            match res {
                Some(hit) => {
                    let abs = hit.hit.get_absolute_position();

                    if btn == MouseButton::Left {
                        lock.chunk_manager.action_queue.break_block(abs);
                    }
                    else if btn == MouseButton::Right {
                        let normal = hit.normal;
                        let target_block_pos = abs + normal;

                        let target_block = Box::new(StoneBlock::new(
                            Vector3::new(target_block_pos.x.rem_euclid(16) as u32, target_block_pos.y.rem_euclid(16) as u32, target_block_pos.z.rem_euclid(16) as u32),
                            target_block_pos
                        ));

                        lock.chunk_manager.action_queue.place_block(target_block);
                    }
                },
                None => {
                    
                }
            }
            //println doesn't flush in another thread...
        });
    }

    {
        let light = gamewindow.renderer.create_spotlight(
            Point3::new(15., 25., 15.), 
            Point3::new(0., 0., 0.)
        );
    }

    let test_table = Table::new("test-table".to_string(), "hello!".to_string());

    {gamewindow.screenui.write().unwrap().add_child(test_table);}

    let textbutton = TextButton::new("button!".to_owned(), "Hello Me!".to_owned());

    textbutton.write().unwrap().on_click.connect(|v| {
        println!("HELLO");
    });

    let (chunksend, chunkget): (
        Sender<(i32, i32, u32, HashMap<u32, ChunkGridType>)>,
        Receiver<(i32, i32, u32, HashMap<u32, ChunkGridType>)>
    ) = mpsc::channel();

    let (meshedsend, meshedget): (
        Sender<(i32, i32, u32, ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32)))>,
        Receiver<(i32, i32, u32, ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32)))>
    ) = mpsc::channel();
    
    {
        let chunk_update_thread = std::thread::spawn(move || {
            println!("IN");
            while let Ok((chunk_x, chunk_z, y_slice, chunks)) = chunkget.recv() {
                let result = mesh_slice_arrayed(chunk_x, chunk_z, y_slice, &chunks);

                meshedsend.send((chunk_x, chunk_z, y_slice, result)).unwrap();
            }
            println!("OUT");
        });
    }

    {
        let wa = workspace_arc.clone();
        workspace.input_service.on_key_pressed.connect(move |(code, _)| {
            
            let lock = &mut wa.write().unwrap().input_service;
            if code == KeyCode::KeyX {
                match lock.get_mouse_lock_state() {
                    MouseLockState::Free => {
                        lock.set_mouse_lock_state(MouseLockState::LockCenter);
                        lock.set_mouse_visible(false);
                    },
                    MouseLockState::Contained => {
                        lock.set_mouse_lock_state(MouseLockState::LockCenter);
                        lock.set_mouse_visible(false);
                    },
                    MouseLockState::LockCenter => {
                        lock.set_mouse_lock_state(MouseLockState::Contained);
                        lock.set_mouse_visible(true);
                    },
                }
            }
        });
    }

    let slider = Slider::new("SLIDERRR".to_owned());

    {gamewindow.screenui.write().unwrap().add_child(text_label);}
    //{gamewindow.screenui.write().unwrap().add_child(textbutton);}
    //{gamewindow.screenui.write().unwrap().add_child(slider);}

    let mut last_update = instant::Instant::now();

    drop(workspace);

    let _ = event_loop.run(
        move |event, control_flow| 
        match event {
            Event::DeviceEvent { device_id, event: DeviceEvent::Key(k) } => {
                
            },
            Event::DeviceEvent {event: DeviceEvent::MouseMotion { delta }, device_id } => {
                let mut workspace = workspace_arc.write().unwrap();
                workspace.current_camera.controller.process_mouse_input(delta.0, delta.1);
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id == window.id() {
                    let consumed = gamewindow.gui_renderer.handle_input(gamewindow.window.clone(), event);
                    let mut workspace = workspace_arc.write().unwrap();
                    match event {
                        WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                            match event.physical_key {
                                PhysicalKey::Code(v) => {
                                    if !consumed {
                                        workspace.current_camera.controller.process_keyboard_input(v, event.state);
                                    }
                                },
                                _ => {}
                            }
                            workspace.input_service.process_key_input(event, consumed).block_on();
                        },
                        WindowEvent::MouseInput { device_id, state, button } => {
                            workspace.input_service.process_mouse_input(button, state, consumed).block_on();
                        },
                        WindowEvent::CloseRequested => {
                            control_flow.exit()
                        },
                        WindowEvent::Resized(physical_size) => {
                            //win.resize(*physical_size);
                        },
                        WindowEvent::RedrawRequested => {
                            if window_id == window.id() {
                                let now = instant::Instant::now();
                                let dt = now - last_update;
                                last_update = now;
                                gamewindow.on_next_frame(&mut workspace, dt.as_secs_f32());
                                workspace.chunk_manager.on_frame_action(&gamewindow.device, &chunksend);
                                workspace.input_service.update();

                                
                                if let Ok(res) = meshedget.try_recv() {
                                    let t = Stopwatch::start_new();
                                    workspace.chunk_manager.finalize_mesh(res.0, res.1, res.2, &gamewindow.device, res.3);
                                    println!("{}", t);
                                }
                            }
                        }
                        _ => {}
                    };
                    //ctx.handle_input(&mut state.window, &event);
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            } _ => {}
        }
    );
}
