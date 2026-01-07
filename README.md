# SkyCanvas
Long exposure photographs with automated Ardupilot Quad(s).



----
# Planning 

## Stage 1 MVP

### Goals
- Proof of concept / light weight prototype of long exposure drone photography tool
- Give a interface to upload a SVG (or similar) path and have a quadcopter fly said path in the flight region
### Requirements
- Run on a single laptop 
- Ardupilot runs on quad w/ remote telementry link to laptop running main stack
- Data logging / viz for debugging
- SIL testing
- User can upload a SVG path and have a quadcopter fly said path in the flight region
- External control method for ground station like control 
### Design
- Stack:
  - Python / UV based prototype
  - Rerun for dataviz
  - Ardupilot based quadcopter w/ mavlink for comms
  - Single / near-single application for all purposes 
    - Sub tasks 
  - Docker for wrapping
- Remote Control:
  - HTTP API for requesting data such as ardupilot fields / parameters
  - HTTP API for sending commands such as ARM or our high level control commands
  - CLI based control app / TUI (bonus / optional) using said HTTP API for Stage 1 demo.
    - Vibe coded GCS using API spec ok for this version
- Main Application:
  - Python application running tassk such as the main ardupilot loop, our web API, our custom logic
  - Shared state (thread locked ideally) of the quad
    - Can either choose a shared mutex state or a message queue / table based approach
    - Probably shared mutex state is easier for this version
  - UPDATE: Message based.
    - Best matches ardupilot
    - Rerun can log to convert to time series
    - All tasks really only care about the message and not the state
    - Can just be an array per task in the task manager w/ sub system
    - This allows for easier testing and debugging
    - Commands are now easier to add in
      - tasks just listen for when a command is not null and runs.
    - In theory nothing is stateful. (tasks can store their own state or we can at least store last values)
    - Maybe a PUB/SUB/GET/SET Model?
- Logging:
  - Embedded as a task is a Rerun based loggering logging at every tick (or event) 
- Autpilot and link
  - Ardupilot based quadcopter
  - Wifi for development but idelaly 900mhz or ELRS based telementry link for test.
  - Connects to laptop over mavlink.
  - Can SIL with docker container w/ full stack for accurate testing
- Camera controll   
  - Need a way to actually do lon exposed
  - Amazon a controller
  - Design a tool + PCB to do this if im cool
  - RP2350 time? 

### Notes

### TODO
- [ ] Setup base application w/ task loop
- [ ] Setup Ardupulot connection w/ SIL Container
- [ ] Setup heartbeat task for Ardupilot using task data (message based)
- [ ] Setup rerun w/ hello world logging of ardupilot data