

pub struct ShowLine{
    id: u32,
    points: Vec<ShowPoint>,
}

impl ShowLine{
    pub fn new(points: Vec<ShowPoint>) -> Self{
        let id = rand::thread_rng().gen_range(0..u32::MAX);
        Self{id, points}    
    }
}