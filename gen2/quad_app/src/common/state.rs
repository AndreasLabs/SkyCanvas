use crate::common::led::LED;
use crate::common::mavlink_helpers::EkfStatus;
#[derive(Default, Debug, Clone)]
pub struct LLA {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
}
#[derive(Default, Debug, Clone)]
pub struct NED {
    pub north: f32,
    pub east: f32,
    pub down: f32,
}

impl NED {
    pub fn new(north: f32, east: f32, down: f32) -> Self {
        Self { north, east, down }
    }
    pub fn distance(&self, other: &NED) -> f32 {
        ((self.north - other.north).powi(2)
            + (self.east - other.east).powi(2)
            + (self.down - other.down).powi(2))
        .sqrt()
    }
}

impl LLA {
    pub fn new(latitude: f32, longitude: f32, altitude: f32) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }
}

const MIN_DISTANCE_TO_RECORD_NED: f32 = 0.01;

#[derive(Default, Debug, Clone)]
pub struct QuadAppState {
    pub status_message: Option<String>,

    pub lla_current: LLA,
    pub ned_current: NED,
    pub ned_history: Vec<NED>,

    pub ekf_status: EkfStatus,

    pub led_state: LED,
}

impl QuadAppState {
    pub fn new() -> Self {
        Self {
            status_message: None,
            lla_current: LLA::default(),
            ned_current: NED::default(),
            ned_history: Vec::new(),
            ekf_status: EkfStatus::default(),
            led_state: LED::default(),
        }
    }

    pub fn record_ned(&mut self, ned: NED) {
        self.ned_current = ned;

        // Only save if the NED is at least 0.01m away from the last entry
        if self.ned_history.len() > 0 {
            let last_ned = self.ned_history.last().unwrap();
            if last_ned.distance(&self.ned_current) > MIN_DISTANCE_TO_RECORD_NED {
                self.ned_history.push(self.ned_current.clone());
            }
        }
    }

    pub fn record_lla(&mut self, lla: LLA) {
        self.lla_current = lla;
    }
}
