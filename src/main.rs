use std::collections::HashMap;
use std::sync::Arc;

use cgmath::{Point3, Vector3};
use gen::primitive::PrimitiveBuilder;
use gui::elements::slider::Slider;
use gui::elements::table::Table;
use gui::elements::textbutton::TextButton;
use internal::window::GameWindow;
use pollster::FutureExt;
use state::workspace::Workspace;
use stopwatch::Stopwatch;
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
    let event_loop = EventLoop::new().unwrap();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    
    let mut gamewindow = GameWindow::new(window.clone()).block_on();

    let mut workspace = Workspace::new(
        &gamewindow.device, &gamewindow.camera_bindgroup_layout, 
        gamewindow.window_size.width, gamewindow.window_size.height,
        window.clone()
    );

    workspace.chunk_manager.generate_chunks(&gamewindow.device);
    workspace.chunk_manager.mesh_chunks(&gamewindow.device);

    {
        let light = gamewindow.renderer.create_spotlight(
            Point3::new(15., 25., 15.), 
            Point3::new(0., 0., 0.)
        );
    }

    let test_map: HashMap<String, String> = HashMap::new();

    let test_table = Table::new("test-table".to_string(), "hello!".to_string());

    {gamewindow.screenui.write().unwrap().add_child(test_table);}

    let textbutton = TextButton::new("button!".to_owned(), "Hello Me!".to_owned());

    textbutton.write().unwrap().on_click.connect(|v| Box::pin(async {
        println!("HELLO");
    }));

    let slider = Slider::new("SLIDERRR".to_owned());

    {gamewindow.screenui.write().unwrap().add_child(textbutton);}
    {gamewindow.screenui.write().unwrap().add_child(slider);}

    let mut last_update = instant::Instant::now();

    let _ = event_loop.run(
        move |event, control_flow| 
        match event {
            Event::DeviceEvent { device_id, event: DeviceEvent::Key(k) } => {
                
            },
            Event::DeviceEvent {event: DeviceEvent::MouseMotion { delta }, device_id } => {
                workspace.current_camera.controller.process_mouse_input(delta.0, delta.1);
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id == window.id() {

                    let consumed = gamewindow.gui_renderer.handle_input(gamewindow.window.clone(), event);

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
