use crate::design::primitives::ShowPrimitive;


pub trait ShowDesignGenerator{
    fn get_name(&self) -> &str;
    fn get_json_schema(&self) -> &str;
    fn generate(&self, json_params: &str) ->Result<ShowDesign, anyhow::Error>;
}


#[allow(dead_code)]
pub struct ShowDesign{
    primitives: Vec<ShowPrimitive>,
}

impl ShowDesign{
    pub fn new() -> Self{
        Self{primitives: vec![]}
    }
}