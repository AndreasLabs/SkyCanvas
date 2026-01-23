#[derive(Default, Debug, Clone)]
pub struct LED{
    pub rgb: [u8; 3],
    pub brightness: f32,
    pub is_on: bool,
}


impl LED{
    pub fn new(rgb: [u8; 3], brightness: f32, is_on: bool) -> Self {
        Self { rgb, brightness, is_on }
    }

    pub fn to_rerun_color(&self) -> rerun::components::Color {
        rerun::components::Color::new([self.rgb[0], self.rgb[1], self.rgb[2]])
    }
}

