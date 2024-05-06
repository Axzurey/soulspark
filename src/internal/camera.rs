use std::f32::consts::FRAC_PI_2;

use cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector2, Vector3};
use instant::Duration;
use winit::{event::ElementState, keyboard::KeyCode};

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Camera {
    pub position: Point3<f32>,

    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,

    view_proj_matrix: Matrix4<f32>,

    yaw: Rad<f32>,
    pitch: Rad<f32>,
    
    fov: Rad<f32>,
    aspect_ratio: f32,
    znear: f32,
    zfar: f32,

    controller: CameraController
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Into<CameraUniform> for Camera {
    fn into(self) -> CameraUniform {
        CameraUniform {
            view_position: [self.position.x, self.position.y, self.position.z, 1.0],
            view_proj: self.view_proj_matrix.into()
        }
    }
}

impl Camera {
    pub fn new<P: Into<Point3<f32>>>(
        position: P,
        yaw: f32,
        pitch: f32,
        aspect_ratio: f32,
        fov: f32
    ) -> Self {
        Self {
            position: position.into(),
            
            pitch: Rad(pitch),
            yaw: Rad(yaw),
            aspect_ratio,
            fov: Rad(fov),
            
            znear: 0.1,
            zfar: 200.,

            projection_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            view_proj_matrix: Matrix4::identity(),
            controller: CameraController::new()
        }
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = Rad(fov);
    }
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect_ratio = aspect;
    }

    pub fn update_matrices(&mut self) {
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();

        self.view_matrix = Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                pitch_cos * yaw_cos,
                pitch_sin,
                pitch_cos * yaw_sin
            ).normalize(),
            Vector3::unit_y()
        );

        self.projection_matrix = OPENGL_TO_WGPU_MATRIX * perspective(self.fov, self.aspect_ratio, self.znear, self.zfar);
    
        self.view_proj_matrix = self.projection_matrix * self.view_matrix;
    }

    pub fn look_vector(&self) -> Vector3<f32> {
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();

        Vector3::new(cos_yaw * cos_pitch, sin_pitch, sin_yaw * cos_pitch).normalize()
    }

    pub fn right_vector(&self) -> Vector3<f32> {
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Vector3::new(-sin_yaw, 0.0, cos_yaw).normalize()
    }

    pub fn up_vector(&self) -> Vector3<f32> {
        self.look_vector().cross(self.right_vector())
    }
}

pub struct CameraController {
    move_delta: Vector3<f32>,
    horizontal_sensitivity: f32,
    vertical_sensitivity: f32,
    mouse_delta: Vector2<f32>,
    space_down: bool,
    shift_down: bool
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            horizontal_sensitivity: 1.0,
            vertical_sensitivity: 1.0,
            move_delta: Vector3::new(0.0, 0.0, 0.0),
            mouse_delta: Vector2::new(0.0, 0.0),
            space_down: false,
            shift_down: false
        }
    }

    pub fn process_keyboard_input(&mut self, key: KeyCode, state: ElementState) -> bool {
        let increment = if state == ElementState::Pressed {1.0} else {0.0};

        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.move_delta.z = increment;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.move_delta.z = -increment;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.move_delta.x = increment;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.move_delta.x = -increment;
                true
            }
            KeyCode::Space => {
                self.space_down = state == ElementState::Pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.shift_down = state == ElementState::Pressed;
                true
            }
            _ => false
        }
    }

    pub fn process_mouse_input(&mut self, dx: f64, dy: f64) {
        self.mouse_delta = Vector2::new(dx as f32, dy as f32);
    }

    pub fn look_vector(&self, camera: &mut Camera) -> Vector3<f32> {
        let (sin_yaw, cos_yaw) = camera.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = camera.pitch.0.sin_cos();

        Vector3::new(cos_yaw * cos_pitch, sin_pitch, sin_yaw * cos_pitch).normalize()
    }

    pub fn right_vector(&self, camera: &mut Camera) -> Vector3<f32> {
        let (sin_yaw, cos_yaw) = camera.yaw.0.sin_cos();

        Vector3::new(-sin_yaw, 0.0, cos_yaw).normalize()
    }

    pub fn up_vector(&self, camera: &mut Camera) -> Vector3<f32> {
        self.look_vector(camera).cross(self.right_vector(camera))
    }
    
    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        //temporary, before i make player controller

        let lv = self.look_vector(camera);
        let rv = self.right_vector(camera);

        camera.position += lv * self.move_delta.z * dt * 10.0;
        camera.position += rv * self.move_delta.x * dt * 10.0;

        camera.yaw += Rad(self.mouse_delta.x) * self.horizontal_sensitivity * dt;
        camera.pitch += Rad(-self.mouse_delta.y) * self.vertical_sensitivity * dt;

        if self.space_down {
            camera.position += Vector3::new(0.0, 10.0 * dt, 0.0);
        }
        if self.shift_down {
            camera.position -= Vector3::new(0.0, 10.0 * dt, 0.0);
        }

        self.mouse_delta.x = 0.0;
        self.mouse_delta.y = 0.0;

        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        }
        else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}