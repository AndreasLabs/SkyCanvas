use std::time::Instant;

use crate::{app::systems::AppSystemTrait, common::{state::NED, waypoint::Waypoint}};

pub enum WaypointState{
    HOLD = 0,
    COMMAND = 1,
    TRANSIT = 2,
    COMPLETE = 3, // PReviously Reached
}
pub struct WaypointSystem{
    path: Vec<Waypoint>,
    current_waypoint: Option<Waypoint>,
    next_waypoint: Option<Waypoint>,
    time_start_hold_ms: Option<u64>,
    state: WaypointState,
    offboard_active: bool,
    last_position_ned: Option<NED>,
    is_enabled: bool,
}

impl WaypointSystem{
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            current_waypoint: None,
            next_waypoint: None,
            time_start_hold_ms: None,
            state: WaypointState::HOLD,
            offboard_active: false,
            last_position_ned: None,
            is_enabled: false,
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.path.push(waypoint);
    }


    pub fn run_path(&mut self, path: Vec<Waypoint>) {
        self.path = path;
        self.is_enabled = true;
    }

}

impl AppSystemTrait for WaypointSystem{
    fn start(&mut self, context: &crate::common::context::QuadAppContext) -> Result<(), anyhow::Error> {
        self.is_enabled = true;
        Ok(())
    }
    fn tick(&mut self, context: &crate::common::context::QuadAppContext) -> Result<(), anyhow::Error> {
        self.tick_state_machine(context)?;
        Ok(())
    }
}

// Tick Functions

impl WaypointSystem{
    fn tick_state_machine(&mut self, context: &crate::common::context::QuadAppContext) -> Result<(), anyhow::Error> {
        match self.state {
            WaypointState::HOLD => self.tick_hold(context)?,
            WaypointState::COMMAND => self.tick_command(context)?,
            WaypointState::TRANSIT => self.tick_transit(context)?,
            WaypointState::COMPLETE => self.tick_complete(context)?,
        }
        Ok(())
    }

    fn tick_hold(&mut self, context: &crate::common::context::QuadAppContext) -> Result<(), anyhow::Error> {
        if !self.is_enabled {
            log::warn!("WaypointSystem // HOLD - Not enabled");
            return Ok(());
        }
        // Check if there are any waypoints in the path
        if self.path.is_empty() {
            self.is_enabled = false;
            log::warn!(
                "WaypointSystem // HOLD - Path complete, disabling automatic processing"
            );
            return Ok(());
        }
        // Pull the next waypoint from the path (index 0)
        self.current_waypoint = Some(self.path.remove(0).clone());
        if self.path.is_empty() {
            self.next_waypoint = None;
        } else {
            self.next_waypoint = Some(self.path[0].clone());
        }
        log::info!(
            "WaypointSystem // HOLD - Pulled next waypoint from path ({})",
            self.path.len()
        );

        // Transition to COMMAND
        self.state = WaypointState::COMMAND;
        Ok(())
    }

    fn tick_command(&mut self, context: &crate::common::context::QuadAppContext) -> Result<(), anyhow::Error> {
        log::info!("WaypointSystem // COMMAND - Starting offboard mode");
        // Set initial setpoint to target position
        let current_waypoint = self.current_waypoint.as_ref().unwrap().clone();
        let target_ned = NED::new(
            current_waypoint.ned.north,
            current_waypoint.ned.east,
            current_waypoint.ned.down,
        );

        Ok(())
    }

    fn tick_transit(&mut self, context: &crate::common::context::QuadAppContext) -> Result<(), anyhow::Error> {
        log::info!("WaypointSystem // TRANSIT - Transiting to next waypoint");
        Ok(())
    }

    fn tick_complete(&mut self, context: &crate::common::context::QuadAppContext) -> Result<(), anyhow::Error> {
        log::info!("WaypointSystem // COMPLETE - Waypoint complete");
        Ok(())
    }
}