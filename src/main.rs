use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use parking_lot::RwLock;
use blocks::stoneblock::StoneBlock;
use cgmath::{Point3, Vector2, Vector3};
use engine::surfacevertex::SurfaceVertex;
use gen::primitive::PrimitiveBuilder;
use gui::elements::slider::Slider;
use gui::elements::textbutton::TextButton;
use gui::elements::textlabel::TextLabel;
use gui::uistate::MouseButton;
use internal::raycaster::raycast_blocks;
use internal::window::GameWindow;
use pollster::FutureExt;
use state::workspace::Workspace;
use stopwatch::Stopwatch;
use util::debugger::Debugger;
use util::inputservice::{InputService, MouseLockState};
use vox::chunk::{xz_to_index, Chunk, ChunkGridType, ChunkState};
use vox::chunk_manager::mesh_slice_arrayed;
use vox::chunkactionqueue::ChunkAction;
use vox::structure_loader::load_structures;
use vox::worker_threads::{spawn_chunk_creation_loop, spawn_chunk_meshing_loop};
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

#[tokio::main()]
async fn main() {

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        for deadlock in parking_lot::deadlock::check_deadlock() {
            for deadlock in deadlock {
                println!(
                    "Found a deadlock! {}:\n{:?}",
                    deadlock.thread_id(),
                    deadlock.backtrace()
                );
            }
        }
    });
    //find way to edit arc

    let event_loop = EventLoop::new().unwrap();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    load_structures();
    
    let mut gamewindow = GameWindow::new(window.clone()).block_on();

    let workspace_arc = Arc::new(RwLock::new(Workspace::new(
        &gamewindow.device, &gamewindow.camera_bindgroup_layout, 
        gamewindow.window_size.width, gamewindow.window_size.height,
        window.clone()
    )));

    let mut workspace = workspace_arc.write();

    let (sendmesh, getmesh) = spawn_chunk_meshing_loop(3);
    let (sendchunk, getchunk) = spawn_chunk_creation_loop(4, workspace.chunk_manager.seed);
    workspace.chunk_manager.generate_chunks(&gamewindow.device, &sendchunk, Vector2::new(0., 0.));

    {
        let wa = workspace_arc.clone();

        workspace.input_service.on_mouse_click.connect(move |(btn, _)| {
            let lock = &mut wa.write();

            let p = lock.current_camera.position;

            let pos = Vector3::new(p.x, p.y, p.z);
            
            let res = raycast_blocks(pos, lock.current_camera.look_vector(), 400.0, &lock.chunk_manager, |_| false);
            
            if res.is_none() {return};

            let r = res.unwrap();

            let abs = r.hit.get_absolute_position();
            let normal = r.normal;

            drop(r);

            if btn == MouseButton::Left {
                lock.chunk_manager.action_queue.break_block(abs);
            }
            else if btn == MouseButton::Right {
                let target_block_pos = abs + normal;

                let target_block = Box::new(StoneBlock::new(
                    Vector3::new(target_block_pos.x.rem_euclid(16) as u32, target_block_pos.y.rem_euclid(16) as u32, target_block_pos.z.rem_euclid(16) as u32),
                    target_block_pos
                ));

                lock.chunk_manager.action_queue.place_block(target_block);
            }
        });
    }

    {
        let light = gamewindow.renderer.create_spotlight(
            Point3::new(15., 25., 15.), 
            Point3::new(0., 0., 0.)
        );
    }

    let mut debugger = Debugger::new(workspace.chunk_manager.seed, &mut gamewindow.screenui);


    {
        let wa = workspace_arc.clone();
        workspace.input_service.on_key_pressed.connect(move |(code, _)| {
            
            let lock = &mut wa.write().input_service;
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

    let mut last_update = instant::Instant::now();

    drop(workspace);

    let _ = event_loop.run(
        move |event, control_flow| 
        match event {
            Event::DeviceEvent { device_id, event: DeviceEvent::Key(k) } => {
                
            },
            Event::DeviceEvent {event: DeviceEvent::MouseMotion { delta }, device_id } => {
                let mut workspace = workspace_arc.write();
                workspace.current_camera.controller.process_mouse_input(delta.0, delta.1);
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id == window.id() {
                    let consumed = gamewindow.gui_renderer.handle_input(gamewindow.window.clone(), event);
                    let mut workspace = workspace_arc.write();
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
                                let framestart = Stopwatch::start_new();
                                let now = instant::Instant::now();
                                let dt = now - last_update;
                                last_update = now;
                                gamewindow.on_next_frame(&mut workspace, dt.as_secs_f32());
                                
                                workspace.chunk_manager.on_frame_action(&gamewindow.device, &sendmesh);
                                workspace.input_service.update();
                                debugger.update(&workspace, &mut gamewindow.screenui);
                                
                                for _ in 0..10 {
                                    if let Ok(res) = getmesh.try_recv() {
                                        let at = Vector3::new(res.0, res.2 as i32, res.1);
                                        let index = workspace.chunk_manager.unresolved_meshes.iter().position(|p| *p == at);
                                        if let Some(i) = index {
                                            workspace.chunk_manager.unresolved_meshes.swap_remove(i);
                                        }
                                        workspace.chunk_manager.finalize_mesh(res.0, res.1, res.2, &gamewindow.device, res.3);
                                    }
                                    else {
                                        break;
                                    }
                                }
                                loop {
                                    if let Ok(res) = getchunk.try_recv() {
                                        let chunkbuff = workspace.chunk_manager.chunk_buffers.get_mut(&xz_to_index(res.0, res.1)).unwrap();

                                        chunkbuff.set_slice_vertex_buffers(&gamewindow.device);

                                        workspace.chunk_manager.chunks.insert(xz_to_index(res.0, res.1), res.2);

                                        if workspace.chunk_manager.chunks.len() as u32 == (workspace.chunk_manager.render_distance * 2 + 1).pow(2) {
                                            println!("Beginning Illumination");
                                            workspace.chunk_manager.generate_chunk_illumination(&gamewindow.device);
                                            workspace.chunk_manager.mesh_chunks(&gamewindow.device, &sendmesh, Vector3::new(0., 0., 0.));
                                        }
                                    }
                                    else {
                                        break;
                                    }
                                }
                                //println!("Frame time: {}ms", framestart.elapsed_ms());
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
