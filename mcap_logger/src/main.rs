use std::{collections::BTreeMap, fs, io::BufWriter, path::{Path, PathBuf}, time::{SystemTime, Duration}};
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use pretty_env_logger;
use tokio::signal;
use clap::Parser;
use mcap::{records::MessageHeader, Writer};
use futures_util::StreamExt;
use chrono::{DateTime, Utc, Local};

// Import from conductor crate
use conductor::redis::{RedisConnection, RedisOptions};

#[derive(Parser, Debug)]
#[clap(author, version, about = "MCAP Logger for Redis messages")]
struct Args {
    /// Redis host
    #[clap(long, default_value = "127.0.0.1")]
    redis_host: String,

    /// Redis port
    #[clap(long)]
    redis_port: Option<u16>,

    /// Redis password
    #[clap(long)]
    redis_password: Option<String>,

    /// Output file path
    #[clap(long, default_value = "output.mcap")]
    output: String,

    /// Redis channel pattern to subscribe to
    #[clap(long, default_value = "*")]
    channel_pattern: String,
    
    /// Enable log rolling after specified minutes (0 = disabled)
    #[clap(long, default_value = "0")]
    roll_minutes: u64,
}

/// Enum representing the different rotation intervals
enum RollInterval {
    Never,
    Minutes(u64),
}

impl RollInterval {
    fn from_minutes(minutes: u64) -> Self {
        if minutes == 0 {
            RollInterval::Never
        } else {
            RollInterval::Minutes(minutes)
        }
    }
    
    fn as_duration(&self) -> Option<Duration> {
        match self {
            RollInterval::Never => None,
            RollInterval::Minutes(mins) => Some(Duration::from_secs(mins * 60)),
        }
    }
}

/// Creates a writer for a new MCAP file
fn create_mcap_writer<P: AsRef<Path>>(path: P) -> Result<Writer<BufWriter<fs::File>>> {
    let file = fs::File::create(path).context("Failed to create output file")?;
    let writer = BufWriter::new(file);
    let mcap_writer = Writer::new(writer)?;
    Ok(mcap_writer)
}

/// Generates a filename with timestamp
fn generate_filename(base_path: &Path, timestamp: DateTime<Local>) -> PathBuf {
    let stem = base_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    
    let extension = base_path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("mcap");
    
    let parent = base_path.parent().unwrap_or(Path::new("."));
    
    let timestamp_str = timestamp.format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("{}-{}.{}", stem, timestamp_str, extension);
    
    parent.join(filename)
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    let redis_options = RedisOptions {
        host: args.redis_host,
        port: args.redis_port,
        password: args.redis_password,
    };
    
    // Determine rolling interval
    let roll_interval = RollInterval::from_minutes(args.roll_minutes);
    
    // Get the base output path
    let base_output_path = Path::new(&args.output);
    
    // Start with the original output path
    let mut current_output_path = if matches!(roll_interval, RollInterval::Never) {
        base_output_path.to_path_buf()
    } else {
        // If rolling is enabled, start with a timestamped file
        generate_filename(base_output_path, Local::now())
    };
    
    // Create MCAP writer
    info!("Creating MCAP file at {}", current_output_path.display());
    let mut mcap_writer = create_mcap_writer(&current_output_path)?;
    
    // Create a channel for each Redis channel/topic we encounter
    let mut channel_map = std::collections::HashMap::new();
    
    // Connect to Redis using conductor's RedisConnection
    let mut redis_conn = RedisConnection::new(redis_options, "mcap_logger".to_string());
    
    // Create pubsub connection
    info!("Subscribing to channel pattern: {}", args.channel_pattern);
    let mut pubsub = redis_conn.client.get_async_pubsub().await?;
    pubsub.psubscribe(&args.channel_pattern).await?;
    
    let mut stream = pubsub.into_on_message();
    
    // Sequence counter for messages
    let mut sequence = 0;
    
    // Track when to roll the log file
    let mut next_roll_time: Option<SystemTime> = None;
    
    if let Some(duration) = roll_interval.as_duration() {
        info!("Log rolling enabled. Will create a new log file every {} minutes", duration.as_secs() / 60);
    } else {
        info!("Log rolling disabled");
    }
    
    info!("MCAP Logger started. Press Ctrl+C to stop and save the file.");
    
    // Set up Ctrl+C handler
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);
    
    loop {
        // Check if it's time to roll the log file
        if let Some(roll_time) = next_roll_time {
            if SystemTime::now() >= roll_time {
                // Finish current log file
                info!("Rolling log file - closing current file");
                mcap_writer.finish()?;
                
                // Create new log file with timestamp
                let new_path = generate_filename(base_output_path, Local::now());
                info!("Creating new log file at {}", new_path.display());
                mcap_writer = create_mcap_writer(&new_path)?;
                current_output_path = new_path;
                
                // Reset channel map as we need to recreate channels in the new file
                channel_map.clear();
                
                // Set next roll time
                if let Some(duration) = roll_interval.as_duration() {
                    next_roll_time = Some(SystemTime::now() + duration);
                    info!("Next log roll scheduled at {}", DateTime::<Local>::from(next_roll_time.unwrap()).format("%Y-%m-%d %H:%M:%S"));
                }
            }
        }
        
        tokio::select! {
            // Handle Redis message
            Some(msg) = stream.next() => {
                let redis_channel: String = msg.get_channel()?;
                let payload: String = msg.get_payload()?;
                
                debug!("Received message on channel '{}': {}", redis_channel, payload);
                
                // If this is our first message and rolling is enabled, set the next roll time
                if next_roll_time.is_none() && matches!(roll_interval, RollInterval::Minutes(_)) {
                    if let Some(duration) = roll_interval.as_duration() {
                        next_roll_time = Some(SystemTime::now() + duration);
                        info!("First message received. Next log roll scheduled at {}", 
                              DateTime::<Local>::from(next_roll_time.unwrap()).format("%Y-%m-%d %H:%M:%S"));
                    }
                }
                
                // Get or create a channel ID for this Redis channel
                let channel_id = if let Some(&id) = channel_map.get(&redis_channel) {
                    id
                } else {
                    // Create a new channel for this Redis channel
                    // Using schema ID 0 for schemaless JSON
                    let new_id = mcap_writer.add_channel(
                        0, // Schema ID 0 - schemaless JSON
                        &redis_channel, // Use Redis channel as topic
                        "json", // Use plain "json" not "application/json"
                        &BTreeMap::new(),
                    )?;
                    channel_map.insert(redis_channel.clone(), new_id);
                    new_id
                };
                
                // Get current time for the message
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                
                // Create message header
                let header = MessageHeader {
                    channel_id,
                    sequence,
                    log_time: now,
                    publish_time: now,
                };
                
                // Store the message in MCAP file
                mcap_writer.write_to_known_channel(&header, payload.as_bytes())?;
                
                // Log timestamp for reference
                let datetime: DateTime<Utc> = SystemTime::now().into();
                info!("Saved message from '{}' at {} (seq: {})", 
                      redis_channel, 
                      datetime.format("%Y-%m-%d %H:%M:%S%.3f UTC"), 
                      sequence);
                
                sequence += 1;
            }
            
            // Handle Ctrl+C
            _ = &mut ctrl_c => {
                info!("Received Ctrl+C, finishing MCAP file...");
                break;
            }
        }
    }
    
    // Finish the MCAP file
    mcap_writer.finish()?;
    info!("MCAP file saved to {}", current_output_path.display());
    
    Ok(())
}
