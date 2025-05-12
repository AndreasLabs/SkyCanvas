
pub trait ShowDesignGenerator{
    fn get_name(&self) -> &str;
    fn get_json_schema(&self) -> &str;
    fn generate(&self, json_params: &str) ->Result<ShowDesign>;
}


pub struct ShowDesign{
    primitives: Vec<ShowPrimitive>,
}
