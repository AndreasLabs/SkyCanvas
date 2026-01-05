use rand::Rng;

use crate::types::{light_color::ShowLightColor, position::ShowPosition};



#[allow(dead_code)]
pub struct ShowPoint{
    id: u32,
    position: ShowPosition,
    color: ShowLightColor,
}

impl ShowPoint{
    pub fn new(position: ShowPosition, color: ShowLightColor) -> Self{
        let id = rand::rng().random_range(0..u32::MAX);
        Self{id, position, color}
    }
}