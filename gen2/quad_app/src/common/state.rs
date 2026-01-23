#[derive(Default, Debug, Clone)]
pub struct LLA{
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}
#[derive(Default, Debug, Clone)]
pub struct NED{
    pub north: f64,
    pub east: f64,
    pub down: f64,
}


impl NED{
    pub fn new(north: f64, east: f64, down: f64) -> Self {
        Self { north, east, down }
    }
}

impl LLA{
    pub fn new(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Self { latitude, longitude, altitude }
    }
}
#[derive(Default, Debug, Clone)]
pub struct QuadAppState{
    pub status_message: Option<String>,

    pub current_lla: LLA,
    pub lla_history: Vec<LLA>,
    pub ned_current: NED,
    pub ned_history: Vec<NED>,
}

impl QuadAppState{
    pub fn new() -> Self {
        Self { status_message: None, current_lla: LLA::default(), lla_history: Vec::new(), ned_current: NED::default(), ned_history: Vec::new() }
    }
}