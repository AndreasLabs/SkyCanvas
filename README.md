# SkyCanvas
Long exposure photographs with automated Ardupilot Quad(s).


# Setup


# Components

## Conductor
- 

## Docker
- Location of the docker compose and build files used
- Has devcontainer
- Has Services for:
    - Ardupilot SIL
    - Redis / Redis Insight

## GroundLink
- Redis <-> WebSocket Bridge for WebClients

## MCAP Logger
- Connects to Redis and logs to MCAP
- Can be used with Foxglove for data viz
- Phasing out in favor of Rerun

## Scenarios
- Rust-based "Scenario" scripts that run commands and a given setup for SIL based testing
- Scenarios are ran via extending a `Scenario` trait and defining what redis commands to send at what `t` time
- Setup of connections to redis are handled be the parent runner.

## Scripts

## Showkit

## ShowMan

 
# Stage 1 MVP (Jan 1st -> Feb 1st 2026)
## Goals
- Basic clean and lightweight repository to work with
- Display an image -> Drone long exposure photography
- Basic SIL and Validation pipeline - for development

## TODO
- [ ] Clean up Repo
    - [x] Get full repo to build
    - [ ] Update README w/ components
- [ ] Showkit Importer
    - [ ] Basic ShowKit Grid Demo
    - [ ] Basic SVG Importer Demo (Pixel Based)
- [ ] Clean build system 
- [ ] Rerun Logger
    - [ ] Move loggers to sub directory