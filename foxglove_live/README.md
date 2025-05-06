# Foxglove Live

A WebSocket server that connects Redis messages to Foxglove Studio for live data visualization.

## Features

- Implements the [Foxglove WebSocket Protocol](https://github.com/foxglove/ws-protocol)
- Automatically discovers Redis channels and creates Foxglove topics
- Dynamically generates JSON schemas from message content
- Supports subscribe/unsubscribe for individual channels
- Handles multiple concurrent WebSocket clients

## Usage

```bash
# Basic usage with default options
cargo run -p foxglove_live

# Custom options
cargo run -p foxglove_live -- --ws-port 8766 --redis-host 192.168.1.100 --channel-pattern "channels/*"
```

## Command Line Options

- `--ws-host`: WebSocket server host (default: 0.0.0.0)
- `--ws-port`: WebSocket server port (default: 8765)
- `--redis-host`: Redis server hostname (default: 127.0.0.1)
- `--redis-port`: Redis server port (optional)
- `--redis-password`: Redis server password (optional)
- `--channel-pattern`: Redis channel pattern to subscribe to (default: *)

## Connecting to Foxglove Studio

1. Open Foxglove Studio (desktop app or web version)
2. Click "Open Connection" and select "Foxglove WebSocket"
3. Enter the WebSocket URL (e.g., `ws://localhost:8765`)
4. Click "Open"

Once connected, you'll see all available topics from Redis channels. Each Redis channel becomes a Foxglove topic with an automatically generated schema.

## Architecture

The server works by:

1. Subscribing to Redis channels matching the provided pattern
2. For each channel, creating a Foxglove topic with a dynamically generated JSON schema
3. Advertising available channels to connected Foxglove clients
4. Forwarding messages from Redis to subscribed WebSocket clients

## Message Format

Redis messages must be valid JSON strings. The server automatically generates JSON schemas based on the structure of the first message received on each channel.

## Dependencies

This project uses:
- The `conductor` crate's Redis utilities for Redis connection
- Tokio and tokio-tungstenite for async WebSocket server
- Serde for JSON serialization/deserialization

## Related Projects

- [mcap_logger](../mcap_logger): Records Redis messages to MCAP files for offline analysis
- [conductor](../conductor): Core libraries including Redis connection utilities 