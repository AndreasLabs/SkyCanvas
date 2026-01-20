# Waypoint Segment ID's
- Currently Waypoint consist of point to point "waypoints" in which the painter UAV
  will only turn on its light brush (Addressable RGBs LEDs) with the desired color+brigtness
  then turn it back off - moving to the next waypoint.
- Ideally to paint smooth "light-painting" style images - we should have the planner / uav support
  "paths" such as lines / splines with the LED continously on and the uav moving during capture - creating
  a painted line streak in the final image.
- One way to achieve - and may be explored later - is to add a line data type as a waypoint type - simialr to current
  point one.
- This proposes a simpler way - by encoding an int index "segment id" that points have as a field.
  - These `segment_id`'s are used to tie related points into a line.
  - Currently connected linearly - but future support could have conntion type to curve be defined.
  - The planner can then see if the next waypoint desired has the same segment_id and use this to decide LED action.
  
  
# Design
- Waypoints get a new uint type of `segment_id` 
- Points that represent a line (linear connection) should have the same `segment_id`
- Once a new - not connected - line is desired - the pattern generator should increment the `segment_id`
- Waypoint system can then use if the next `segment_id` is the same as current to decide:
  - If same: Keep the LED on and depending on a normalize distance - blend the colors and brightness as the UAV travels
    - In the future this can be a configuable option of blending mode to support linear cut offs 
  - If different:
    - Turn off the LED and turn it back on for next waypoint.
