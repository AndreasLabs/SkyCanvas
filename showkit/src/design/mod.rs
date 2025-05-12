pub mod primitives;

pub struct ShowDesign{
    primitives: Vec<ShowPrimitive>,
}

impl ShowDesign{
    pub fn new() -> Self{
        Self{primitives: vec![]}
    }

    pub fn add_primitive(&mut self, primitive: ShowPrimitive){
        self.primitives.push(primitive);
    }
}