#[derive(Default, Debug, Clone)]
pub struct Waypoint{
    pub ned: NED,
    pub color: [u8; 3],
    pub hold_time: f32,
    pub yaw_deg: f32,
    pub segment_id: u32,
}

impl Waypoint{
    pub fn new(ned: NED, color: [u8; 3], hold_time: f32, yaw_deg: f32, segment_id: u32) -> Self {
        Self { ned, color, hold_time, yaw_deg, segment_id }
    }
}