use crate::design::primitives::point::ShowPoint;



pub struct ShowLine{
    id: u32,
    points: Vec<ShowPoint>,
}

impl ShowLine{
    pub fn new(points: Vec<ShowPoint>) -> Self{
        let id = rand::random_range(0..u32::MAX);
        Self{id, points}    
    }
}