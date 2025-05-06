# MCAP Logger

A simple utility to subscribe to Redis channels and log all messages to an MCAP file using the Foxglove-compatible JSON format.

## Features

- Subscribes to all Redis channels or a specific pattern
- Saves messages with proper timestamps
- Creates an individual topic for each Redis channel
- Properly formats messages for Foxglove visualization
- Graceful shutdown with Ctrl+C to properly finish the MCAP file
- Automatic log rolling after a specified time interval

## Usage

```bash
# Subscribe to all Redis channels (default)
cargo run -p mcap_logger

# Use custom options
cargo run -p mcap_logger -- --redis-host 192.168.1.100 --redis-port 6379 --output flight_log.mcap --channel-pattern "channels/*"

# Enable log rolling every 10 minutes
cargo run -p mcap_logger -- --roll-minutes 10
```

## Command Line Options

- `--redis-host`: Redis server hostname (default: 127.0.0.1)
- `--redis-port`: Redis server port (optional)
- `--redis-password`: Redis server password (optional)
- `--output`: Output MCAP file path (default: output.mcap)
- `--channel-pattern`: Redis channel pattern to subscribe to (default: *)
- `--roll-minutes`: Enable log rolling after specified minutes; 0 to disable (default: 0)

## Log Rolling

When log rolling is enabled, the logger will:

1. Create a new timestamped file (e.g., `output-20250506-123045.mcap`) for the initial log
2. After the specified number of minutes, close the current file and create a new timestamped file
3. Continue logging to the new file
4. Repeat this process until the logger is stopped with Ctrl+C

This feature is useful for:
- Preventing log files from becoming too large
- Creating logical segments of data
- Enabling easier troubleshooting by time period

## About MCAP

MCAP is a container file format for multimodal data. It's designed for robotics applications and supports arbitrary message serialization formats. Learn more at [mcap.dev](https://mcap.dev/).

## Foxglove Compatibility

This logger creates MCAP files that are compatible with Foxglove visualization tools. Each Redis channel is recorded as a separate topic with schemaless JSON messages. When viewing in Foxglove, use the Raw Messages panel to inspect the data.

## Dependencies

This project uses:
- The Rust `mcap` library for MCAP file creation
- Redis connection utilities from the `conductor` crate 