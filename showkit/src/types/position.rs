

#[derive(Debug, Clone, Copy)]
pub struct ShowPosition{
    pub local_x: f64,
    pub local_y: f64,
    pub local_z: f64,
}

impl ShowPosition{
    pub fn new(local_x: f64, local_y: f64, local_z: f64) -> Self{
        Self{local_x, local_y, local_z}
    }
}

