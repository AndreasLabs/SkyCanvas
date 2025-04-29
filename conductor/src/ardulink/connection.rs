use anyhow::Error;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, error, info, trace};
use mavlink::ardupilotmega::MavMessage;
use redis::{Commands, PubSub, RedisConnectionInfo};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{self, sync::Mutex, task, time};

use crate::{ardulink::config::ArdulinkConnectionType, redis::RedisConnection, state::State};

type MavlinkMessageType = MavMessage;

#[derive(thiserror::Error, Debug)]
pub enum ArdulinkError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] Error),
    #[error("Channel send error: {0}")]
    ChannelSendError(#[from] crossbeam_channel::SendError<MavlinkMessageType>),
    #[error("Task join error: {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
}

pub struct ArdulinkConnection {
    recv_channels: (Sender<MavlinkMessageType>, Receiver<MavlinkMessageType>),
    transmit_channels: (Sender<MavlinkMessageType>, Receiver<MavlinkMessageType>),
    connection_string: String,
    should_stop: Arc<AtomicBool>,
    connection_type: ArdulinkConnectionType,
    task_handles: Vec<task::JoinHandle<()>>,
    redis: Arc<Mutex<RedisConnection>>,
}

impl ArdulinkConnection {
    pub fn new(connection_type: ArdulinkConnectionType, state: &State) -> Result<Self, Error> {
        let (recv_tx, recv_rx): (Sender<_>, Receiver<_>) = crossbeam_channel::bounded(500);
        let (transmit_tx, transmit_rx): (Sender<_>, Receiver<_>) = crossbeam_channel::bounded(500);
        let redis = RedisConnection::new(state.redis.clone(), "ardulink".to_string());
        let redis = Arc::new(Mutex::new(redis));
        Ok(Self {
            recv_channels: (recv_tx, recv_rx),
            transmit_channels: (transmit_tx, transmit_rx),
            connection_string: connection_type.connection_string(),
            should_stop: Arc::new(AtomicBool::new(false)),
            connection_type,
            task_handles: Vec::new(),
            redis,
        })
    }

    pub async fn start_task(&mut self) -> Result<(), ArdulinkError> {
        let con_string = self.connection_string.clone();
        let recv_channels = self.recv_channels.clone();
        let transmit_channels = self.transmit_channels.clone();
        let should_stop = self.should_stop.clone();
        let connection_type = self.connection_type.clone();
        let redis = self.redis.clone();
        let error_redis = redis.clone();
        let task_handle = task::spawn(async move {
            if let Err(e) = Self::start_task_inner(
                con_string.clone(),
                recv_channels,
                transmit_channels,
                should_stop,
                connection_type,
                redis,
            ).await {
                error!(
                    "ArduLink => Error starting task for connection string: {}",
                    con_string
                );
                error!("ArduLink => Error: {e:?}");

                // Send on ardulink/error channel and log channel
                let mut redis = error_redis.lock().await;
                let _: () = redis.client.publish("ardulink/message", &e.to_string()).unwrap();
                let _: () = redis.client.publish("ardulink/state", &"ERROR").unwrap();
                let _: () = redis.client.publish("log", &format!("ArduLink => Error: {e:?}")).unwrap();
            }
        });

        self.task_handles.push(task_handle);
        Ok(())
    }

    pub async fn stop_task(&mut self) -> Result<(), ArdulinkError> {
        info!("ArduLink => Stopping connection tasks");
        self.should_stop.store(true, Ordering::SeqCst);

        // Wait a bit for tasks to notice the stop flag
        time::sleep(Duration::from_millis(100)).await;

        // Join all tasks
        let handles = std::mem::take(&mut self.task_handles);
        for handle in handles {
            if let Err(e) = handle.await {
                error!("ArduLink => Error joining task: {:?}", e);
            }
        }

        info!("ArduLink => All tasks stopped");
        Ok(())
    }

    async fn start_task_inner(
        con_string: String,
        recv_channels: (Sender<MavlinkMessageType>, Receiver<MavlinkMessageType>),
        transmit_channels: (Sender<MavlinkMessageType>, Receiver<MavlinkMessageType>),
        should_stop: Arc<AtomicBool>,
        _connection_type: ArdulinkConnectionType,
        redis: Arc<Mutex<RedisConnection>>,
    ) -> Result<(), ArdulinkError> {
        // Make the connection
        info!(
            "ArduLink => Connecting to MAVLink with connection string: {}",
            con_string
        );

        let mut mav_con: Box<dyn mavlink::MavConnection<MavlinkMessageType> + Send + Sync> =
            mavlink::connect::<MavlinkMessageType>(&con_string)
                .map_err(|e| ArdulinkError::ConnectionError(e.into()))?;

        info!("ArduLink => Setting up connection parameters");
        mav_con.set_protocol_version(mavlink::MavlinkVersion::V2);

        // Request streams now handled by ExecTaskRequestStream

        let mav_con = Arc::new(mav_con);

        info!("ArduLink => Starting main tasks...");

        // Send task
        info!("ArduLink => Spawning send task");
        let send_handle = task::spawn({
            let vehicle = mav_con.clone();
            let should_stop = should_stop.clone();
            let (_, rx) = transmit_channels;
            async move {
                while !should_stop.load(Ordering::SeqCst) {
                    match rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(msg) => {
                            if should_stop.load(Ordering::SeqCst) {
                                break;
                            }
                            trace!("ArduLink => Sending message to MAVLink: {msg:?}");
                            // Only attempt to send if we're not stopping
                            if !should_stop.load(Ordering::SeqCst) {
                                let _ = vehicle.send(&mavlink::MavHeader::default(), &msg);
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                            // Check if we should stop
                            if should_stop.load(Ordering::SeqCst) {
                                break;
                            }
                            // Allow other tasks to run
                            task::yield_now().await;
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }
                debug!("ArduLink => Send task exiting");
            }
        });

        // Receive task
        info!("ArduLink => Spawning receive task");
        let receive_handle = task::spawn({
            let vehicle = mav_con.clone();
            let should_stop = should_stop.clone();
            let (recv_tx, _) = recv_channels;
            async move {
                while !should_stop.load(Ordering::SeqCst) {
                    if should_stop.load(Ordering::SeqCst) {
                        break;
                    }

                    // Use standard receive with a timeout by checking the flag frequently
                    let recv_result = vehicle.recv();

                    match recv_result {
                        Ok((_header, msg)) => {
                            if let Err(e) = recv_tx.send(msg.clone()) {
                                error!(
                                    "ArduLink => Failed to send received message to channel: {:?}",
                                    e
                                );
                                if should_stop.load(Ordering::SeqCst) {
                                    break;
                                }
                            }

                            let msg_json = serde_json::to_string(&msg).unwrap();

                            let mut redis = redis.lock().await;
                            let _: () = redis.client.publish("ardulink/raw", &msg_json).unwrap();
                        }
                        Err(mavlink::error::MessageReadError::Io(e)) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                // No messages currently available to receive -- wait a while
                                time::sleep(Duration::from_millis(10)).await;
                            } else if !should_stop.load(Ordering::SeqCst) {
                                // Only log errors if we're not stopping
                                error!("ArduLink => Receive error: {e:?}");
                                break;
                            }
                        }
                        // Messages that didn't get through due to parser errors are ignored
                        _ => {}
                    }

                    // Check stop flag more frequently
                    if should_stop.load(Ordering::SeqCst) {
                        break;
                    }
                    
                    // Allow other tasks to run
                    task::yield_now().await;
                }
                debug!("ArduLink => Receive task exiting");
            }
        });

        // Join tasks when one exits or stop is requested
        let _ = send_handle.await;
        let _ = receive_handle.await;

        info!("ArduLink => All tasks exited");
        Ok(())
    }

    pub fn send(&self, msg: &MavlinkMessageType) -> Result<(), ArdulinkError> {
        // Don't attempt to send if we're stopping
        if self.should_stop.load(Ordering::SeqCst) {
            return Ok(());
        }

        let (tx, _) = &self.transmit_channels;
        tx.send(msg.clone())
            .map_err(ArdulinkError::ChannelSendError)
    }

    pub fn recv(&self) -> Result<Vec<MavlinkMessageType>, ArdulinkError> {
        let mut data = Vec::new();
        let (_, rx) = &self.recv_channels;

        // Don't attempt to receive if we're stopping
        if self.should_stop.load(Ordering::SeqCst) {
            return Ok(data);
        }

        while let Ok(msg) = rx.try_recv() {
            data.push(msg);
        }
        Ok(data)
    }
}
