use anyhow::Error;
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use mavlink::ardupilotmega::MavMessage;
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
        tasks::{task_heartbeat::ArdulinkTask_Heartbeat, task_recv::ArdulinkTask_Recv, task_send::ArdulinkTask_Send, task_request_stream::ArdulinkTask_RequestStream, task_health::ArdulinkTask_Health},
    },
    redis::RedisConnection,
    state::State,
    transformers::{Transformer, TransformerTask},
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
    task_handles: Vec<task::JoinHandle<Result<(), anyhow::Error>>>,
    redis: Arc<Mutex<RedisConnection>>,
    state: State,
    transformers: Vec<Arc<dyn Transformer>>,
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
            transformers: Vec::new(),
        })
    }

    /// Add a transformer to the connection
    pub fn add_transformer(&mut self, transformer: Arc<dyn Transformer>) {
        self.transformers.push(transformer);
    }

    /// Add multiple transformers to the connection
    pub fn add_transformers(&mut self, transformers: Vec<Arc<dyn Transformer>>) {
        self.transformers.extend(transformers);
    }

    pub async fn start_task(&mut self) -> Result<(), ArdulinkError> {
        let con_string = self.connection_string.clone();
        let should_stop = self.should_stop.clone();
        let state = self.state.clone();

        let task_handle = task::spawn(async move {
            let mut mav_con: Box<dyn mavlink::MavConnection<MavlinkMessageType> + Send + Sync> =
            mavlink::connect::<MavlinkMessageType>(&con_string)
                .map_err(|e| ArdulinkError::ConnectionError(e.into()))?;


             // Make the connection
            info!(
                "ArduLink => Connecting to MAVLink with connection string: {}",
                con_string
            );

       

            info!("ArduLink => Setting up connection parameters");
            mav_con.set_protocol_version(mavlink::MavlinkVersion::V2);

        // Request streams now handled by ExecTaskRequestStream

        let mav_con = Arc::new(Mutex::new(mav_con));

            info!("ArduLink => Starting main tasks...");

          
            let receive_handle =
            ArdulinkTask_Recv::spawn(mav_con.clone(), should_stop.clone(), &state).await;

            let send_handle = ArdulinkTask_Send::spawn(mav_con.clone(), should_stop.clone(), &state).await;

            let heartbeat_handle = ArdulinkTask_Heartbeat::spawn(should_stop.clone(), &state).await;

            let request_stream_handle = ArdulinkTask_RequestStream::spawn(should_stop.clone(), &state).await;

            let health_handle = ArdulinkTask_Health::spawn(should_stop.clone(), &state).await;

            // Join tasks when one exits or stop is requested
            let _ = receive_handle.await;
            let _ = send_handle.await;
            let _ = heartbeat_handle.await;
            let _ = request_stream_handle.await;
            let _ = health_handle.await;
            info!("ArduLink => All tasks exited");
            Ok(())


        });

        self.task_handles.push(task_handle);
        
        // If there are transformers, start the transformer task
        if !self.transformers.is_empty() {
            info!("ArduLink => Starting transformer task with {} transformers", self.transformers.len());
            let transformers = self.transformers.clone();
            let should_stop = self.should_stop.clone();
            let state = self.state.clone();
            
            let transformer_task = TransformerTask::spawn(
                transformers,
                should_stop,
                &state
            ).await;
            
            self.task_handles.push(transformer_task);
        }
        
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
