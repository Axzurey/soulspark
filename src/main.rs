use std::sync::Arc;

use cgmath::{Point3, Vector3};
use gen::primitive::PrimitiveBuilder;
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
mod blocks;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    
    let mut gamewindow = GameWindow::new(window.clone()).block_on();

    let mut workspace = Workspace::new(
        &gamewindow.device, &gamewindow.camera_bindgroup_layout, 
        gamewindow.window_size.width, gamewindow.window_size.height
    );

    {
        let light = gamewindow.renderer.create_spotlight(
            Point3::new(15., 25., 15.), 
            Point3::new(0., 0., 0.)
        );

        let read = light.read().unwrap();
        
        // let obj0 = PrimitiveBuilder::new()
        //     .set_diffuse_texture_by_name("grass-top")
        //     .set_primitive(&gamewindow.device, gen::primitive::Primitive::Cube)
        //     .set_size(Vector3::new(5., 5., 5.))
        //     .set_position(Vector3::new(0., 25., 15.))
        //     .finalize();
        // gamewindow.renderer.render_storage.add_object(Arc::new(obj0));

        // let obj1 = PrimitiveBuilder::new()
        //     .set_diffuse_texture_by_name("grass-top")
        //     .set_primitive(&gamewindow.device, gen::primitive::Primitive::Cube)
        //     .set_size(Vector3::new(15., 15., 15.))
        //     .set_position(Vector3::new(0., 0., 0.))
        //     .finalize();
        // gamewindow.renderer.render_storage.add_object(Arc::new(obj1));
    }

    {
        let obj1 = PrimitiveBuilder::new()
            .set_diffuse_texture_by_name("dirt")
            .set_primitive(&gamewindow.device, gen::primitive::Primitive::Cube)
            .set_size(Vector3::new(5., 15., 55.))
            .set_position(Vector3::new(10., 10., 10.))
            .finalize();
        gamewindow.renderer.render_storage.add_object(Arc::new(obj1));

        let obj2 = PrimitiveBuilder::new()
            .set_diffuse_texture_by_name("dirt")
            .set_primitive(&gamewindow.device, gen::primitive::Primitive::Cube)
            .set_size(Vector3::new(1000., 5., 1000.))
            .set_position(Vector3::new(0., -10., 0.))
            .finalize();
        gamewindow.renderer.render_storage.add_object(Arc::new(obj2));

        let obj3 = PrimitiveBuilder::new()
            .set_diffuse_texture_by_name("dirt")
            .set_primitive(&gamewindow.device, gen::primitive::Primitive::Cube)
            .set_size(Vector3::new(15., 15., 15.))
            .set_position(Vector3::new(0., 0., 15.))
            .finalize();
        gamewindow.renderer.render_storage.add_object(Arc::new(obj3));
    }

    let mut last_update = instant::Instant::now();

    let _ = event_loop.run(
        move |event, control_flow| 
        match event {
            Event::DeviceEvent { device_id, event: DeviceEvent::Key(k) } => {
                match k.physical_key {
                    PhysicalKey::Code(v) => {
                        workspace.current_camera.controller.process_keyboard_input(v, k.state);
                    },
                    _ => {}
                }
            },
            Event::DeviceEvent {event: DeviceEvent::MouseMotion { delta }, device_id } => {
                workspace.current_camera.controller.process_mouse_input(delta.0, delta.1);
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                
                
                if window_id == window.id() {
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput {event: KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape), ..
                        }, ..} => {

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
