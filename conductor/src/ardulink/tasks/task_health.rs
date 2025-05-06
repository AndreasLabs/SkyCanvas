use crate::{ardulink::connection::MavlinkConnection, redis::RedisConnection};
use crate::state::State;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use mavlink::ardupilotmega::{EkfStatusFlags, MavMessage, EKF_STATUS_REPORT_DATA, SYS_STATUS_DATA};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::{
    task::{self, JoinHandle},
    time::{self, Duration, Instant},
};
use redis::Commands;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    AWAITING_DATA,
    AWAITING_LOCK,
    HEALTHY,
    UNHEALTHY,
}

pub struct ArdulinkTask_Health {
    // Internal state tracking
    current_status: HealthStatus,
    last_reason: String,
    last_check_time: Instant,
    check_interval: Duration,
    last_update_time: Instant,
    update_interval: Duration,
    // Data tracking
    has_sys_status_data: bool,
    last_sys_status: Option<SYS_STATUS_DATA>,
    has_ekf_data: bool,
    last_ekf_status: Option<EKF_STATUS_REPORT_DATA>,
    // Health flags
    system_healthy: bool,
    ekf_attitude_velocity_ok: bool,
    ekf_position_ok: bool,
}

impl ArdulinkTask_Health {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            current_status: HealthStatus::AWAITING_DATA,
            last_reason: "Initializing health monitor".to_string(),
            last_check_time: now,
            check_interval: Duration::from_millis(500), // Check health state more frequently
            last_update_time: now,
            update_interval: Duration::from_secs(3),    // Publish updates less frequently (every 3 seconds)
            has_sys_status_data: false,
            last_sys_status: None,
            has_ekf_data: false,
            last_ekf_status: None,
            system_healthy: false,
            ekf_attitude_velocity_ok: false,
            ekf_position_ok: false,
        }
    }

    // --- Health Check Logic (inspired by provided examples) ---

    /// Check if system status is healthy
    fn check_system_health(sys_status: &SYS_STATUS_DATA) -> (bool, String) {
        let comms_healthy = sys_status.errors_comm < 100; // Allow some communication errors
        let battery_healthy =
            sys_status.battery_remaining == -1 || sys_status.battery_remaining > 20; // Check if > 20% or not reported

        let overall_healthy = comms_healthy && battery_healthy;
        let reason = if !overall_healthy {
            let mut reasons = Vec::new();
            if !comms_healthy {
                reasons.push(format!("Comm errors: {}", sys_status.errors_comm));
            }
            if !battery_healthy {
                reasons.push(format!("Battery low: {}%", sys_status.battery_remaining));
            }
            format!("System unhealthy: {}", reasons.join(", "))
        } else {
            "System status OK".to_string()
        };

        (overall_healthy, reason)
    }

    /// Check if EKF has attitude and velocity
    fn check_ekf_attitude_velocity(ekf_status: &EKF_STATUS_REPORT_DATA) -> (bool, String) {
        let required_flags = EkfStatusFlags::EKF_ATTITUDE | EkfStatusFlags::EKF_VELOCITY_HORIZ;
        let ok = (ekf_status.flags & required_flags) == required_flags;
        let reason = if ok {
            "EKF attitude/velocity OK".to_string()
        } else {
            format!(
                "EKF attitude/velocity not ready (Flags: {:?})",
                ekf_status.flags
            )
        };
        (ok, reason)
    }

    /// Check if EKF has position lock
    fn check_ekf_position(ekf_status: &EKF_STATUS_REPORT_DATA) -> (bool, String) {
        // Check if any horizontal position flag is set
        let horiz_pos_flags = EkfStatusFlags::EKF_POS_HORIZ_REL | EkfStatusFlags::EKF_POS_HORIZ_ABS;
        let has_horiz_pos = (ekf_status.flags & horiz_pos_flags).bits() > 0;
        // Also require vertical position
        let has_vert_pos = (ekf_status.flags & EkfStatusFlags::EKF_POS_VERT_ABS).bits() > 0;

        let ok = has_horiz_pos && has_vert_pos;
        let reason = if ok {
            "EKF position lock OK".to_string()
        } else {
             format!(
                "EKF position lock not ready (Flags: {:?})",
                ekf_status.flags
            )
        };
        (ok, reason)
    }


    // --- State Update Logic ---
    fn update_health_status(&mut self) -> (HealthStatus, String) {
         let mut current_reason = Vec::<String>::new(); // Explicitly make this Vec<String>

         // Start with AWAITING_DATA
         if !self.has_sys_status_data || !self.has_ekf_data {
             if !self.has_sys_status_data { current_reason.push("Waiting for SYS_STATUS".to_string()); }
             if !self.has_ekf_data { current_reason.push("Waiting for EKF_STATUS_REPORT".to_string()); }
             return (HealthStatus::AWAITING_DATA, current_reason.join("; "));
         }

         // Check System Health first
         if let Some(sys_status) = &self.last_sys_status {
             let (healthy, reason) = Self::check_system_health(sys_status);
             self.system_healthy = healthy;
              if !healthy {
                 current_reason.push(reason);
                 return (HealthStatus::UNHEALTHY, current_reason.join("; "));
              } else {
                 current_reason.push("System OK".to_string());
              }
         } else {
             // Should not happen if has_sys_status_data is true, but handle defensively
             return (HealthStatus::AWAITING_DATA, "Missing SYS_STATUS data".to_string());
         }

        // Check EKF Attitude/Velocity
        if let Some(ekf_status) = &self.last_ekf_status {
             let (ok, reason) = Self::check_ekf_attitude_velocity(ekf_status);
             self.ekf_attitude_velocity_ok = ok;
             if !ok {
                current_reason.push(reason);
                // Still need basic EKF attitude/velocity for AWAITING_LOCK
                return (HealthStatus::AWAITING_LOCK, current_reason.join("; "));
             } else {
                current_reason.push("EKF Att/Vel OK".to_string());
             }

            // Check EKF Position Lock (only if attitude/velocity is ok)
            let (ok, reason) = Self::check_ekf_position(ekf_status);
            self.ekf_position_ok = ok;
            if !ok {
                current_reason.push(reason);
                // If system is healthy and EKF has attitude/velocity but no position lock -> AWAITING_LOCK
                return (HealthStatus::AWAITING_LOCK, current_reason.join("; "));
            } else {
                 current_reason.push("EKF Pos OK".to_string());
            }
        } else {
             // Should not happen if has_ekf_data is true
             return (HealthStatus::AWAITING_DATA, "Missing EKF_STATUS data".to_string());
        }

        // If all checks passed
        (HealthStatus::HEALTHY, "System healthy and EKF locked".to_string())
    }


    // --- Task Spawn ---
    pub async fn spawn(
        should_stop: Arc<AtomicBool>,
        state: &State,
    ) -> JoinHandle<Result<(), anyhow::Error>> {
        info!("ArduLink // HealthTask // Spawning");
        let state = state.clone();


        task::spawn(async move {
            let mut health_state = ArdulinkTask_Health::new();
            let mut redis = RedisConnection::new(state.redis.clone(), "ardulink_health".to_string());
            
            // Publish initial status
            let initial_status_json = serde_json::to_string(&health_state.current_status)?;
            let _: () = redis.client.publish("ardulink/health/status", &initial_status_json)?;
            let _: () = redis.client.publish("ardulink/health/reason", &health_state.last_reason)?;

            // Subscribe to MAVLink channels
            let (mut redis_sink, mut redis_stream) = redis.client.get_async_pubsub().await?.split();
            redis_sink.subscribe("channels/ardulink/recv/SYS_STATUS").await?;
            redis_sink.subscribe("channels/ardulink/recv/EKF_STATUS_REPORT").await?;

            info!("ArduLink // HealthTask // Subscribed to SYS_STATUS and EKF_STATUS_REPORT channels");

            while !should_stop.load(Ordering::SeqCst) {
                tokio::select! {
                    Some(msg) = redis_stream.next() => {
                        let payload: String = match msg.get_payload() {
                            Ok(p) => p,
                            Err(e) => {
                                error!("ArduLink // HealthTask // Failed to get payload: {}", e);
                                continue;
                            }
                        };
                        let channel_name = msg.get_channel_name();
                        // Attempt to deserialize based on channel
                        match channel_name {
                            "channels/ardulink/recv/SYS_STATUS" => {
                                match serde_json::from_str::<MavMessage>(&payload) {
                                    Ok(MavMessage::SYS_STATUS(data)) => {
                                        trace!("ArduLink // HealthTask // Received SYS_STATUS: {:?}", data);
                                        health_state.has_sys_status_data = true;
                                        health_state.last_sys_status = Some(data);
                                    },
                                    Ok(_) => trace!(
                                        "ArduLink // HealthTask // Received non-SYS_STATUS message on SYS_STATUS channel"
                                    ),
                                    Err(e) => warn!("ArduLink // HealthTask // Failed to deserialize SYS_STATUS from payload '{}': {}", payload, e),
                                }
                            },
                            "channels/ardulink/recv/EKF_STATUS_REPORT" => {
                                match serde_json::from_str::<MavMessage>(&payload) {
                                     Ok(MavMessage::EKF_STATUS_REPORT(data)) => {
                                        trace!("ArduLink // HealthTask // Received EKF_STATUS_REPORT: {:?}", data);
                                        health_state.has_ekf_data = true;
                                        health_state.last_ekf_status = Some(data);
                                    },
                                    Ok(_) => trace!(
                                        "ArduLink // HealthTask // Received non-EKF_STATUS_REPORT message on EKF_STATUS_REPORT channel"
                                    ),
                                    Err(e) => warn!("ArduLink // HealthTask // Failed to deserialize EKF_STATUS_REPORT from payload '{}': {}", payload, e),
                                }
                            },
                            _ => {
                                trace!("ArduLink // HealthTask // Ignoring message from channel: {}", channel_name);
                            }
                        }
                        
                        // Recalculate health after receiving new data
                        health_state.last_check_time = Instant::now();
                        let (new_status, new_reason) = health_state.update_health_status();
                        
                        // Publish updates if:
                        // 1. Status or reason changed, OR
                        // 2. It's been longer than update_interval since last update
                        let should_update = 
                            new_status != health_state.current_status || 
                            new_reason != health_state.last_reason ||
                            health_state.last_update_time.elapsed() >= health_state.update_interval;
                            
                        if should_update {
                            if new_status != health_state.current_status || new_reason != health_state.last_reason {
                                info!("ArduLink // HealthTask // Status changed: {:?} -> {:?}, Reason: {}", 
                                      health_state.current_status, new_status, &new_reason);
                            } else {
                                debug!("ArduLink // HealthTask // Periodic status update: {:?}, Reason: {}", 
                                       new_status, &new_reason);
                            }
                            
                            health_state.current_status = new_status;
                            health_state.last_reason = new_reason;
                            health_state.last_update_time = Instant::now();

                            // Publish updated status
                            let status_json = serde_json::to_string(&health_state.current_status)?;
                            let _: () = redis.client.publish("ardulink/health/status", &status_json)?;
                            let _: () = redis.client.publish("ardulink/health/reason", &health_state.last_reason)?;
                        }
                    }
                    _ = time::sleep_until(health_state.last_check_time + health_state.check_interval) => {
                        // Periodically re-evaluate health even if no new messages arrive
                        trace!("ArduLink // HealthTask // Periodic check");
                        let (new_status, new_reason) = health_state.update_health_status();

                        // Publish updates if:
                        // 1. Status or reason changed, OR
                        // 2. It's been longer than update_interval since last update
                        let should_update = 
                            new_status != health_state.current_status || 
                            new_reason != health_state.last_reason ||
                            health_state.last_update_time.elapsed() >= health_state.update_interval;
                            
                        if should_update {
                            if new_status != health_state.current_status || new_reason != health_state.last_reason {
                                info!("ArduLink // HealthTask // Status changed (periodic): {:?} -> {:?}, Reason: {}", 
                                      health_state.current_status, new_status, &new_reason);
                            } else {
                                debug!("ArduLink // HealthTask // Periodic status update: {:?}, Reason: {}", 
                                       new_status, &new_reason);
                            }
                            
                            health_state.current_status = new_status;
                            health_state.last_reason = new_reason;
                            health_state.last_update_time = Instant::now();

                            // Publish updated status
                            let status_json = serde_json::to_string(&health_state.current_status)?;
                            let _: () = redis.client.publish("ardulink/health/status", &status_json)?;
                            let _: () = redis.client.publish("ardulink/health/reason", &health_state.last_reason)?;
                        }
                    }
                    else => {
                        // Stream closed or stop signal received
                        if should_stop.load(Ordering::SeqCst) {
                             info!("ArduLink // HealthTask // Stop signal received.");
                        } else {
                             warn!("ArduLink // HealthTask // PubSub stream unexpectedly closed.");
                        }
                        break;
                    }
                }
            }

            debug!("ArduLink // HealthTask // Exiting");
            Ok(())
        })
    }
}
