#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rgb<T = f32> {
    r: T,
    g: T,
    b: T,
}
impl<T> Rgb<T> {
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
    pub fn alpha(self, a: T) -> Rgba<T> {
        Rgba::new(self.r, self.g, self.b, a)
    }
}
impl Default for Rgb {
    fn default() -> Self {
        Rgb::new(0.0, 0.0, 0.0)
    }
}
impl From<wavefront_obj::mtl::Color> for Rgb {
    fn from(f: wavefront_obj::mtl::Color) -> Self {
        Self {
            r: f.r as f32,
            g: f.g as f32,
            b: f.b as f32,
        }
    }
}
impl<T: Copy> From<[T; 3]> for Rgb<T> {
    fn from(f: [T; 3]) -> Self {
        Self {
            r: f[0],
            g: f[1],
            b: f[2],
        }
    }
}
impl<T> Into<[T; 3]> for Rgb<T> {
    fn into(self) -> [T; 3] {
        [self.r, self.g, self.b]
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rgba<T = f32> {
    r: T,
    g: T,
    b: T,
    a: T,
}
impl<T> Rgba<T> {
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }
    pub fn rgb(self) -> Rgb<T> {
        Rgb::new(self.r, self.g, self.b)
    }
}
impl Default for Rgba {
    fn default() -> Self {
        Rgba::new(0.0, 0.0, 0.0, 0.0)
    }
}
impl<T: Copy> From<[T; 4]> for Rgba<T> {
    fn from(f: [T; 4]) -> Self {
        Self {
            r: f[0],
            g: f[1],
            b: f[2],
            a: f[3],
        }
    }
}
impl<T> Into<[T; 4]> for Rgba<T> {
    fn into(self) -> [T; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
