use std::f32::consts::FRAC_PI_2;

use cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Transform, Vector2, Vector3};
use instant::Duration;
use wgpu::util::DeviceExt;
use winit::{event::ElementState, keyboard::KeyCode};

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub const OPENGL_TO_WGPU_MATRIX_GLAM: glam::Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Camera {
    pub position: Point3<f32>,

    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,

    view_proj_matrix: Matrix4<f32>,
    inv_view_proj_matrix: Matrix4<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    
    pub fov: Rad<f32>,
    pub aspect_ratio: f32,
    pub screendims: (u32, u32),
    znear: f32,
    zfar: f32,

    pub controller: CameraController,
    buffer: wgpu::Buffer,
    pub bindgroup: wgpu::BindGroup
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    screendims: [u32; 4]
}

impl Into<CameraUniform> for Camera {
    fn into(self) -> CameraUniform {
        CameraUniform {
            view_position: [self.position.x, self.position.y, self.position.z, 1.0],
            view_proj: self.view_proj_matrix.into(),
            inv_view_proj: self.inv_view_proj_matrix.into(),
            screendims: [0, 0, 0, 0]
        }
    }
}

impl Camera {
    pub fn new<P: Into<Point3<f32>>>(
        position: P,
        yaw: f32,
        pitch: f32,
        aspect_ratio: f32,
        fov: f32,
        device: &wgpu::Device,
        bindgroup_layout: &wgpu::BindGroupLayout,
        dims: (u32, u32)
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform {
                view_position: [0., 0., 0., 1.],
                view_proj: Matrix4::from_nonuniform_scale(0., 0., 0.).into(),
                inv_view_proj: Matrix4::from_nonuniform_scale(0., 0., 0.).into(),
                screendims: [dims.0, dims.1, 0, 0]
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                },
            ],
            label: Some("Camera bind group :)")
        });

        Self {
            position: position.into(),
            
            pitch: Rad(pitch.to_radians()),
            yaw: Rad(yaw.to_radians()),
            aspect_ratio,
            fov: Rad(fov.to_radians()),
            
            znear: 0.1,
            zfar: 400.,

            projection_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            view_proj_matrix: Matrix4::identity(),
            inv_view_proj_matrix: Matrix4::identity(),
            controller: CameraController::new(),

            buffer,
            bindgroup,
            screendims: dims
        }
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = Rad(fov);
    }
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect_ratio = aspect;
    }

    pub fn update_matrices(&mut self, queue: &wgpu::Queue) {
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

        self.inv_view_proj_matrix = self.view_proj_matrix.inverse_transform().unwrap();

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[CameraUniform {
            view_position: self.position.to_homogeneous().into(),
            view_proj: self.view_proj_matrix.into(),
            inv_view_proj: self.inv_view_proj_matrix.into(),
            screendims: [self.screendims.0, self.screendims.1, 0, 0]
        }]));
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

    pub fn update_camera(&mut self, dt: f32) {

        let lv = self.look_vector();
        let rv = self.right_vector();

        self.position += lv * self.controller.move_delta.z * dt * 10.0;
        self.position += rv * self.controller.move_delta.x * dt * 10.0;

        self.yaw += Rad(self.controller.mouse_delta.x) * self.controller.horizontal_sensitivity * dt;
        self.pitch += Rad(-self.controller.mouse_delta.y) * self.controller.vertical_sensitivity * dt;

        if self.controller.space_down {
            self.position += Vector3::new(0.0, 10.0 * dt, 0.0);
        }
        if self.controller.shift_down {
            self.position -= Vector3::new(0.0, 10.0 * dt, 0.0);
        }

        self.controller.mouse_delta.x = 0.0;
        self.controller.mouse_delta.y = 0.0;

        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        }
        else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }
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
}