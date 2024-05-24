use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use cgmath::{Point3, Vector3};
use gen::primitive::PrimitiveBuilder;
use gui::elements::slider::Slider;
use gui::elements::table::Table;
use gui::elements::textbutton::TextButton;
use gui::elements::textlabel::TextLabel;
use internal::raycaster::raycast_blocks;
use internal::window::GameWindow;
use pollster::FutureExt;
use state::workspace::Workspace;
use stopwatch::Stopwatch;
use util::inputservice::{InputService, MouseLockState};
use vox::chunk::xz_to_index;
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
    let event_loop = EventLoop::new().unwrap();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    
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
            println!("Hello world!");
            let lock = &mut wa.write().unwrap();

            let p = lock.current_camera.position;

            let pos = Vector3::new(p.x, p.y, p.z);
            
            let res = raycast_blocks(pos, lock.current_camera.look_vector(), 400.0, &lock.chunk_manager, |_| false);
        
            match res {
                Some(hit) => {
                    cloned_label.write().unwrap().set_text(hit.hit.get_name());
                    println!("{:?}", hit.hit.get_name());

                    let abs = hit.hit.get_absolute_position();

                    
                },
                None => {
                    cloned_label.write().unwrap().set_text("NOTHING HIT".to_owned());
                    println!("Nothing");
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
                                workspace.input_service.update();
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
