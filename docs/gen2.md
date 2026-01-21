# Gen 2

## Quick Notes / Ideas

- Rust rewrite of quad_app.py
- Try and use MAVLINNK raw instead of MAVSDK for more control and less repeated async wait behaviour

## Pipeline (proposal)

```

                                            RwLock<State>
                                            Queue<MavCommands>
                                                  I
[MavlinkIO] -> <MsgName, MavlinkMessage> -> [MavTasks]
            <- <MsgName, MavlinkMessage> <-

---- ^ QuadLink -- + -- QuadApp v -----  == Quad

                  RwLock<State>
                Queue<MavCommands>
                      I
[Quad] -> Tick -> [Systems[]]
```

## Quad App Systems:

- Startup (one-shot)
- Led Control (100hz)
- Heartbeat (2hz)
- Waypoint System (100hz)
- TakeOff (one-shot)
-
