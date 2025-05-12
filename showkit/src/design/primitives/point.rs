

pub struct ShowPoint{
    id: u32,
    position: ShowPosition,
    color: ShowLightColor,
}

impl ShowPoint{
    pub fn new(position: ShowPosition, color: ShowLightColor) -> Self{
        let id = rand::thread_rng().gen_range(0..u32::MAX);
        Self{id, position, color}
    }
}