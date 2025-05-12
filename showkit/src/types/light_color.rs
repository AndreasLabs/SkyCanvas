
#[derive(Debug, Clone, Copy)]
pub struct ShowLightColor{
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl ShowLightColor{
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self{
        Self{r, g, b, a}
    }

    pub fn from_rgb(r: f64, g: f64, b: f64) -> Self{
        Self{r, g, b, a: 1.0}
    }

    pub fn from_rgba(r: f64, g: f64, b: f64, a: f64) -> Self{
        Self{r, g, b, a}
    }

}