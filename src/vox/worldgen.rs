use std::sync::RwLock;

use noise::{NoiseFn, Perlin};
use once_cell::sync::Lazy;
use splines::{Interpolation, Key, Spline};

static SPLINE_CAVE_Y_MOD: Lazy<Spline<f32, f32>> = Lazy::new(|| {
    Spline::from_vec(vec![
        Key::new(0.0, 0., Interpolation::Linear),
        Key::new(6., 0.5, Interpolation::Linear),
        Key::new(16., 0.5, Interpolation::Linear),
        Key::new(50., 0.4, Interpolation::Linear),
        Key::new(64., 0.1, Interpolation::Linear),
        Key::new(256., 0.0, Interpolation::Linear),
    ])
});

static SPLINE_WORM: Lazy<Spline<f32, f32>> = Lazy::new(|| {
    Spline::from_vec(vec![
        Key::new(0., 100., Interpolation::Linear),
        Key::new(6., 0.7, Interpolation::Linear),
        Key::new(15., 0.7, Interpolation::Linear),
        Key::new(50., 0.8, Interpolation::Linear),
        Key::new(60., 10.9, Interpolation::Linear),
        Key::new(64., 2.0, Interpolation::Linear),
        Key::new(80., 2.1, Interpolation::Linear),
        Key::new(150., 2.3, Interpolation::Linear),
        Key::new(256., 100.0, Interpolation::Linear),
    ])
});

static SPLINE_CONTINENTALNESS: Lazy<Spline<f32, f32>> = Lazy::new(|| {
    Spline::from_vec(vec![
        Key::new(0.0, 43.0, Interpolation::Linear),
        Key::new(0.3, 61.0, Interpolation::Linear),
        Key::new(0.4, 62.0, Interpolation::Linear),
        Key::new(0.5, 64.0, Interpolation::Linear),
        Key::new(0.6, 110.0, Interpolation::Linear),
        Key::new(0.7, 120.0, Interpolation::Linear),
        Key::new(1.0, 170.0, Interpolation::Linear),
    ])
});

static SPLINE_PEAKS: Lazy<Spline<f32, f32>> = Lazy::new(|| {
    Spline::from_vec(vec![
        Key::new(0.0, 0.0, Interpolation::Linear),
        Key::new(0.3, 0.0, Interpolation::Linear),
        Key::new(0.6, 1.5, Interpolation::Linear),
        Key::new(0.7, 2.0, Interpolation::Linear),
        Key::new(0.85, 6.0, Interpolation::Linear),
        Key::new(1.0, 7.0, Interpolation::Linear)
    ])
});

static SPLINE_FLATNESS: Lazy<Spline<f32, f32>> = Lazy::new(|| {
    Spline::from_vec(vec![
        Key::new(0.0, 0.0, Interpolation::Linear),
        Key::new(0.3, 0.05, Interpolation::Linear),
        Key::new(0.6, 0.01, Interpolation::Linear),
        Key::new(0.7, 00.9, Interpolation::Linear),
        Key::new(1.0, 1.0, Interpolation::Linear),
    ])
});
#[inline]
pub fn perlin_octaved_3d(perlin: Perlin, x: i32, y: i32, z: i32, octaves: i32, mut amp: f32, mut freq: f32, persistence_a: f32, persistence_f: f32, zoom: f32) -> f32 {
    let mut total: f32 = 0.0;
    let mut amp_sum: f32 = 0.0;

    let zoom_inverse = 1. / zoom;

    for i in 0..octaves {
        let v = perlin.get([
            ((x as f32) * zoom_inverse * freq) as f64, ((y as f32) * zoom_inverse * freq) as f64, ((z as f32) * zoom_inverse * freq) as f64
        ]).clamp(-1.0, 1.0) * (amp as f64);

        total += v as f32;
        amp_sum += amp;
        amp *= persistence_a;
        freq *= persistence_f;
    }

    total / amp_sum
}
#[inline]
pub fn perlin_octaved_2d(perlin: Perlin, x: i32, z: i32, octaves: i32, mut amp: f32, mut freq: f32, persistence_a: f32, persistence_f: f32, zoom: f32) -> f32 {
    let mut total: f32 = 0.0;
    let mut amp_sum: f32 = 0.0;

    for i in 0..octaves {
        let v = perlin.get([
            ((x as f32) / zoom * freq) as f64, ((z as f32) / zoom * freq) as f64
        ]).clamp(-1.0, 1.0) * (amp as f64);

        total += v as f32;
        amp_sum += amp;
        amp *= persistence_a;
        freq *= persistence_f;
    }

    total / amp_sum
}
#[inline]
pub fn get_modifiers(noisegen: Perlin, x: i32, z: i32) -> [f32; 3] {
    let continentalness = perlin_octaved_2d(noisegen, x, z, 6, 1.3, 1.2, 0.2, 2.0, 400.0) * 0.5 + 0.5;
    let flatness = perlin_octaved_2d(noisegen, x, z, 6, 0.7, 1.0, 0.2, 2.0, 400.0).abs();
    let peaks = perlin_octaved_2d(noisegen, x, z, 6, 1.5, 1.3, 0.2, 2.0, 400.0) * 0.5 + 0.5;

    [continentalness, flatness, peaks]
}
#[inline]
pub fn is_cave(noisegen: Perlin, x: i32, y: i32, z: i32) -> bool {
    (1. - perlin_octaved_3d(noisegen, x, y, z, 1, 1.3, 1.4, 0.5, 0.5, 35.).abs()) 
        * SPLINE_WORM.sample(y as f32).expect(&format!("y is {}", y)) <= 0.5 
    //|| !get_density_for_cave(noisegen, x, y, z)
}
#[inline]
pub fn get_density_for_cave(noisegen: Perlin, x: i32, y: i32, z: i32) -> bool {
    let p = perlin_octaved_3d(noisegen, x, y, z, 1, 1.1, 1.35, 0.6, 1.1, 1.) * 10.;

    p * SPLINE_CAVE_Y_MOD.sample(y as f32).unwrap() < 1.
}
#[inline]
pub fn density_map_plane(noisegen: Perlin, x: i32, z: i32) -> bool {
    let noise = perlin_octaved_2d(noisegen, x, z, 1, 1.3, 0.7, 0.2, 0.5, 25.0);

    let noise1 = perlin_octaved_2d(noisegen, x + 1, z, 1, 1.3, 0.7, 0.2, 0.5, 25.0);
    let noise2 = perlin_octaved_2d(noisegen, x - 1, z, 1, 1.3, 0.7, 0.2, 0.5, 25.0);
    let noise3 = perlin_octaved_2d(noisegen, x, z + 1, 1, 1.3, 0.7, 0.2, 0.5, 25.0);
    let noise4 = perlin_octaved_2d(noisegen, x, z - 1, 1, 1.3, 0.7, 0.2, 0.5, 25.0);

    noise < noise1 && noise < noise2 && noise < noise3 && noise < noise4
}
#[inline]
pub fn generate_surface_height(noisegen: Perlin, x: i32, z: i32) -> i32 {

    let [c, f, p] = get_modifiers(noisegen, x, z);

    let [cz, fz, pz] = [
        SPLINE_CONTINENTALNESS.sample(c).unwrap(),
        SPLINE_FLATNESS.sample(f).unwrap(),
        SPLINE_PEAKS.sample(p).unwrap()
    ];

    let mut height = cz;

    height += perlin_octaved_2d(noisegen, x, z, 6, 1.1, 
    1.3, 0.2, 2.0,
     75.
    ) 
    * (20.) * pz * (1.0 - fz);

    height.round() as i32
}