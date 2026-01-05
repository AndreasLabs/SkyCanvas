use crate::design::primitives::{line::ShowLine, point::ShowPoint};

pub mod line;
pub mod point;

pub enum ShowPrimitive{
    Point(ShowPoint),
    Line(ShowLine),
}