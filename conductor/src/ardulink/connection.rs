use anyhow::Error;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, error, info, trace};
use mavlink::ardupilotmega::MavMessage;
use redis::{Commands, PubSub, RedisConnectionInfo};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{self, sync::Mutex, task, time};

use crate::{
    ardulink::{
        config::ArdulinkConnectionType,
        tasks::{task_recv::ArdulinkTask_Recv, task_send::ArdulinkTask_Send},
    },
    redis::RedisConnection,
    state::State,
};

type MavlinkMessageType = MavMessage;

pub type MavlinkConnection =
    Arc<Mutex<Box<dyn mavlink::MavConnection<MavlinkMessageType> + Send + Sync>>>;

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
    state: State,
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
            state: state.clone(),
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
        let state = self.state.clone();
        let task_handle = task::spawn(async move {
            if let Err(e) = Self::start_task_inner(
                con_string.clone(),
                recv_channels,
                transmit_channels,
                should_stop,
                connection_type,
                redis,
                state,
            )
            .await
            {
                error!(
                    "ArduLink => Error starting task for connection string: {}",
                    con_string
                );
                error!("ArduLink => Error: {e:?}");

                // Send on ardulink/error channel and log channel
                let mut redis = error_redis.lock().await;
                let _: () = redis
                    .client
                    .publish("ardulink/message", &e.to_string())
                    .unwrap();
                let _: () = redis.client.publish("ardulink/state", &"ERROR").unwrap();
                let _: () = redis
                    .client
                    .publish("log", &format!("ArduLink => Error: {e:?}"))
                    .unwrap();
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
        state: State,
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

        let mav_con = Arc::new(Mutex::new(mav_con));

        info!("ArduLink => Starting main tasks...");

        let receive_handle =
            ArdulinkTask_Recv::spawn(mav_con.clone(), should_stop.clone(), &state).await;

        // Join tasks when one exits or stop is requested
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
