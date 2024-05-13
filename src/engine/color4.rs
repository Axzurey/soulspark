#[derive(Clone, Copy, PartialEq)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Into<[u8; 4]> for Color4 {
    fn into(self) -> [u8; 4] {
        [(self.r * 255.) as u8, (self.g * 255.) as u8, (self.b * 255.) as u8, (self.a * 255.) as u8]
    }
}

impl Into<[f32; 4]> for Color4 {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Color4 {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r, g, b, a
        }
    }
    pub fn from_urgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: (r / 255) as f32,
            g: (g / 255) as f32,
            b: (b / 255) as f32,
            a: (a / 255) as f32
        }
    }
}