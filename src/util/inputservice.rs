use std::{collections::{HashMap, VecDeque}, sync::{Arc, RwLock}};

use winit::{dpi::PhysicalPosition, event::{KeyEvent, RawKeyEvent}, keyboard::KeyCode, window::Window};

use crate::gui::uistate::MouseButton;

use super::threadsignal::MonoThreadSignal;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MouseLockState {
    Free,
    Contained,
    LockCenter 
}

#[derive(Copy, Clone)]
enum InputActionQueue {
    MouseVisible(bool),
    MouseLockState(MouseLockState)
}

pub struct InputService {
    pub on_key_pressed: MonoThreadSignal<KeyCode>,
    pub on_key_released: MonoThreadSignal<KeyCode>,
    /**
     * first parameter: the mouse button
     * second parameter: whether or not the click was absorbed by ui
     */
    pub on_mouse_click: MonoThreadSignal<(MouseButton, bool)>,
    pub on_mouse_move: MonoThreadSignal<(f64, f64)>,
    key_states: HashMap<KeyCode, bool>,

    mouse_lock_state: MouseLockState,
    mouse_visible: bool,
    
    window: Arc<Window>,

    fnqueue: Arc<RwLock<VecDeque<InputActionQueue>>>
}

impl InputService {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            on_key_pressed: MonoThreadSignal::new(),
            on_key_released: MonoThreadSignal::new(),
            on_mouse_click: MonoThreadSignal::new(),
            on_mouse_move: MonoThreadSignal::new(),
            key_states: HashMap::new(),
            mouse_lock_state: MouseLockState::Free,
            mouse_visible: true,
            window,
            fnqueue: Arc::new(RwLock::new(VecDeque::new()))
        }
    }

    pub fn set_mouse_visible(&mut self, visible: bool) {
        self.fnqueue.write().unwrap().push_back(InputActionQueue::MouseVisible(visible));
    }

    fn iset_mouse_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
        self.mouse_visible = visible;
    }

    pub fn get_mouse_visible(&self) -> bool {self.mouse_visible}

    fn iset_mouse_lock_state(&mut self, state: MouseLockState) {
        match state {
            MouseLockState::Free => {
                self.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
            },
            MouseLockState::Contained => {
                self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
            },
            MouseLockState::LockCenter => {
                self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
            },
        };
        self.mouse_lock_state = state;
    }

    pub fn set_mouse_lock_state(&mut self, state: MouseLockState) {
        self.fnqueue.write().unwrap().push_back(InputActionQueue::MouseLockState(state));
    }

    pub fn get_mouse_lock_state(&self) -> MouseLockState {self.mouse_lock_state}

    /**
     * returns whether or not to permit camera motion with that input
     * 
     * possible reasons for returning false can be the MouseLockState being of the Free variant
     */
    pub async fn process_mouse_move(&mut self, delta: (f64, f64)) -> bool {
        if delta.0 != 0. && delta.1 != 0. {
            self.on_mouse_move.dispatch(delta).await;
        }
        match self.mouse_lock_state {
            MouseLockState::Free => {
                false
            },
            MouseLockState::Contained => false,
            MouseLockState::LockCenter => {
                let ws = self.window.inner_size();
                self.window.set_cursor_position(PhysicalPosition::new(ws.width / 2, ws.height / 2)).unwrap();
                true
            },
        }
    }

    pub fn update(&mut self) {
        let mut u = self.fnqueue.write().unwrap();
    
        let mut q = u.clone();

        u.clear();
        drop(u);
        
        loop {
            
            let element = q.pop_front();
            if element.is_none() {break};

            let action = element.unwrap();

            match action {
                InputActionQueue::MouseLockState(s) => self.iset_mouse_lock_state(s),
                InputActionQueue::MouseVisible(v) => self.iset_mouse_visible(v)
            }
        }
    }

    pub async fn process_key_input(&mut self, k: &KeyEvent, consumed: bool) {
        match k.physical_key {
            winit::keyboard::PhysicalKey::Code(code) => {
                match k.state {
                    winit::event::ElementState::Pressed => {
                        if !self.key_states.contains_key(&code) {
                            self.key_states.insert(code, true);
                            self.on_key_pressed.dispatch(code).await;
                        }
                    },
                    winit::event::ElementState::Released => {
                        if self.key_states.contains_key(&code) {
                            self.key_states.remove(&code);
                            self.on_key_released.dispatch(code).await;
                        }
                    },
                }
            },
            winit::keyboard::PhysicalKey::Unidentified(_) => {},
        }
    }
}