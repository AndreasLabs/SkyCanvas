import React, { useState, useEffect, useRef, useCallback } from 'react';
import { 
  Container, 
  Title, 
  Group, 
  Card, 
  Text, 
  Progress, 
  Badge, 
  Stack, 
  ScrollArea, 
  ThemeIcon,
  RingProgress,
  Paper,
  Box,
  SimpleGrid,
  Center,
} from '@mantine/core';
import { 
  IconBattery, 
  IconWifi, 
  IconWifiOff, 
  IconMessage, 
  IconAlertTriangle,
  IconInfoCircle,
  IconGps,
  IconPlaneTilt,
  IconHeartRateMonitor
} from '@tabler/icons-react';
import { WebSocketStatus } from '../hooks/useRedisWebSocket';
import { useRedisStore } from '../store/redisStore';

interface TerrainReport {
  type: 'TERRAIN_REPORT';
  lat: number;
  lon: number;
  terrain_height: number;
  current_height: number;
  spacing: number;
  pending: number;
  loaded: number;
}

interface BatteryStatus {
  type: 'BATTERY_STATUS';
  id: number;
  battery_function: number;
  battery_type: number;
  temperature: number;
  voltages: number[];
  current_battery: number;
  current_consumed: number;
  energy_consumed: number;
  battery_remaining: number;
  time_remaining: number;
  charge_state: number;
  voltages_ext: number[];
  mode: number;
  fault_bitmask: number;
}

interface Attitude {
  type: 'ATTITUDE';
  time_boot_ms: number;
  roll: number;
  pitch: number;
  yaw: number;
  rollspeed: number;
  pitchspeed: number;
  yawspeed: number;
}

interface GlobalPositionInt {
  type: 'GLOBAL_POSITION_INT';
  time_boot_ms: number;
  lat: number; // degrees * 10^7
  lon: number; // degrees * 10^7
  alt: number; // mm
  relative_alt: number; // mm
  vx: number; // cm/s
  vy: number; // cm/s
  vz: number; // cm/s
  hdg: number; // heading in degrees * 100
}

interface StatusText {
  type: 'STATUSTEXT';
  severity: number;
  text: string;
  id: number;
  chunk_seq: number;
}

interface Heartbeat {
  type: 'HEARTBEAT';
  custom_mode: number;
  mavtype: number;
  autopilot: number;
  base_mode: number;
  system_status: number;
  mavlink_version: number;
}

interface SysStatus {
  type: 'SYS_STATUS';
  onboard_control_sensors_present: number;
  onboard_control_sensors_enabled: number;
  onboard_control_sensors_health: number;
  load: number;
  voltage_battery: number;
  current_battery: number;
  battery_remaining: number;
  drop_rate_comm: number;
  errors_comm: number;
  errors_count1: number;
  errors_count2: number;
  errors_count3: number;
  errors_count4: number;
}

const getSeverityColor = (severity: number): string => {
  switch (severity) {
    case 0: // Emergency
    case 1: // Alert
      return 'red';
    case 2: // Critical
    case 3: // Error
      return 'orange';
    case 4: // Warning
      return 'yellow';
    case 5: // Notice
    case 6: // Info
      return 'blue';
    case 7: // Debug
      return 'gray';
    default:
      return 'blue';
  }
};

const getSeverityIcon = (severity: number) => {
  if (severity <= 3) return <IconAlertTriangle size={16} />;
  if (severity <= 4) return <IconInfoCircle size={16} />;
  return <IconMessage size={16} />;
};

const formatLatLon = (value: number): string => {
  return (value / 10000000).toFixed(6);
};

const formatAltitude = (value: number): string => {
  return (value / 1000).toFixed(2);
};

const useThrottledEffect = (callback: () => void, delay: number, deps: React.DependencyList) => {
  const lastRan = useRef(Date.now());
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      const now = Date.now();
      if (now - lastRan.current >= delay) {
        callback();
        lastRan.current = now;
      }
    }, delay - (Date.now() - lastRan.current)); // Adjust timeout based on time already passed
    return () => clearTimeout(timeoutId);
  }, deps);
};

export function ArduPilotGCS() {
  const [terrain, setTerrain] = useState<TerrainReport | null>(null);
  const [battery, setBattery] = useState<BatteryStatus | null>(null);
  const [attitude, setAttitude] = useState<Attitude | null>(null);
  const [position, setPosition] = useState<GlobalPositionInt | null>(null);
  const [statusText, setStatusText] = useState<StatusText[]>([]);
  const [heartbeat, setHeartbeat] = useState<Heartbeat | null>(null);
  const [sysStatus, setSysStatus] = useState<SysStatus | null>(null);
  const [connected, setConnected] = useState(false);
  const [lastHeartbeatTime, setLastHeartbeatTime] = useState<number | null>(null);
  
  const { status, messages, subscribedChannels } = useRedisStore();
  
  // Ref to store the latest message timestamp for each channel to avoid re-processing
  const lastProcessedTimestamps = useRef<Record<string, number>>({});

  // Simplified message processing logic
  const processMavlinkMessage = useCallback((channel: string, content: string, timestamp: number) => {
    if (timestamp <= (lastProcessedTimestamps.current[channel] || 0)) return;
    lastProcessedTimestamps.current[channel] = timestamp;
    
    if (!channel.includes('ardulink/recv')) return;

    try {
      const parsedContent = JSON.parse(content);
      if (!parsedContent || !parsedContent.type) return;

      // Fix lexical declaration error by declaring outside switch
      let newStatusText: StatusText;
      let decodedTextString: string;

      switch (parsedContent.type) {
        case 'TERRAIN_REPORT': setTerrain(parsedContent as TerrainReport); break;
        case 'BATTERY_STATUS': setBattery(parsedContent as BatteryStatus); break;
        case 'ATTITUDE': setAttitude(parsedContent as Attitude); break;
        case 'GLOBAL_POSITION_INT': setPosition(parsedContent as GlobalPositionInt); break;
        case 'STATUSTEXT':
          // Safely decode text
          decodedTextString = '';
          if (typeof parsedContent.text === 'string') {
            decodedTextString = parsedContent.text;
          } else if (Array.isArray(parsedContent.text)) {
            try {
              // Filter and convert character codes
              decodedTextString = String.fromCharCode(...parsedContent.text.filter((code: number) => code > 0 && code < 256));
            } catch (decodeError) {
              console.error("Error decoding status text array:", decodeError);
              decodedTextString = "[Decoding Error]";
            }
          }
          
          // Create the new StatusText object with the decoded string
          newStatusText = {
             type: 'STATUSTEXT', // Explicitly set type
             severity: parsedContent.severity,
             id: parsedContent.id,
             chunk_seq: parsedContent.chunk_seq,
             text: decodedTextString // Assign the decoded string
          };
          
          setStatusText(prev => {
             if (prev.length > 0 && prev[0].text === newStatusText.text) return prev;
             return [newStatusText, ...prev].slice(0, 10);
          });
          break;
        case 'HEARTBEAT':
          setHeartbeat(parsedContent as Heartbeat);
          setLastHeartbeatTime(Date.now());
          setConnected(true);
          break;
        case 'SYS_STATUS':
          setSysStatus(parsedContent as SysStatus);
          break;
      }
    } catch (error) {
      // Use the error variable in the log
       console.error('Error processing message:', error, content);
    }
  }, []);

  // Simplified useEffect to process messages from the store
  useEffect(() => {
    Object.entries(messages).forEach(([channel, latestMessage]) => { // Iterate directly over message objects
      if (latestMessage) { // Check if a message exists for the channel
        // Process the single latest message directly
        processMavlinkMessage(channel, latestMessage.content, latestMessage.timestamp);
      }
    });
  }, [messages, processMavlinkMessage]);

  // Heartbeat monitoring (remains the same)
  useEffect(() => {
    const interval = setInterval(() => {
      if (lastHeartbeatTime) {
        const timeSinceLastHeartbeat = Date.now() - lastHeartbeatTime;
        if (timeSinceLastHeartbeat > 3000) {
          setConnected(false);
          console.log('No heartbeat received for 3+ seconds');
        }
      } else {
        // If we haven't received a heartbeat yet, assume disconnected
        setConnected(false);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [lastHeartbeatTime]);
  
  // Ref for the attitude canvas
  const attitudeCanvasRef = useRef<HTMLCanvasElement>(null);

  // Attitude drawing logic (correct useThrottledEffect arguments)
  useThrottledEffect(() => {
    if (attitudeCanvasRef.current && attitude) {
       const canvas = attitudeCanvasRef.current;
       const ctx = canvas.getContext('2d');
       if (ctx) {
          const width = canvas.width;
          const height = canvas.height;
          const centerX = width / 2;
          const centerY = height / 2;
          const radius = Math.min(width, height) / 2 * 0.8;

          ctx.clearRect(0, 0, width, height);
          ctx.save();
          ctx.translate(centerX, centerY);
          ctx.rotate(attitude.roll);

          const pitchOffset = radius * Math.sin(attitude.pitch);
          ctx.beginPath();
          ctx.moveTo(-radius, pitchOffset);
          ctx.lineTo(radius, pitchOffset);
          ctx.strokeStyle = 'white';
          ctx.lineWidth = 2;
          ctx.stroke();

          ctx.fillStyle = '#7cb6ff'; // Sky
          ctx.fillRect(-radius, -radius, radius * 2, radius + pitchOffset);
          ctx.fillStyle = '#8e7941'; // Ground
          ctx.fillRect(-radius, pitchOffset, radius * 2, radius - pitchOffset);
          
          // Simplified pitch lines
          ctx.strokeStyle = 'white';
          ctx.lineWidth = 1;
          for (let i = -3; i <= 3; i++) {
            if (i === 0) continue;
            const y = pitchOffset - i * (radius / 6); // Adjusted spacing
            const lineLength = radius * 0.2;
            ctx.beginPath();
            ctx.moveTo(-lineLength, y);
            ctx.lineTo(lineLength, y);
            ctx.stroke();
          }

          ctx.restore();

          // Fixed aircraft symbol
          ctx.strokeStyle = 'yellow';
          ctx.lineWidth = 3;
          ctx.beginPath();
          ctx.moveTo(centerX - 30, centerY);
          ctx.lineTo(centerX - 10, centerY);
          ctx.moveTo(centerX + 10, centerY);
          ctx.lineTo(centerX + 30, centerY);
          ctx.moveTo(centerX, centerY - 10);
          ctx.lineTo(centerX, centerY + 10);
          ctx.stroke();
          
          // Simplified heading - just text
          const headingDeg = (attitude.yaw * 180 / Math.PI).toFixed(0);
          ctx.fillStyle = 'white';
          ctx.font = 'bold 16px Arial';
          ctx.textAlign = 'center';
          ctx.fillText(`${headingDeg}°`, centerX, centerY + radius + 35);
       }
    }
  }, 100, [attitude, attitudeCanvasRef]); // Pass correct dependencies

  // Simplified return statement - render directly
  return (
    <Container fluid mt="md">
      <Group justify="space-between" mb="lg">
        <Title order={2}>ArduPilot Ground Control Station</Title>
        <Group>
          <Badge 
            size="lg" 
            color={connected ? 'green' : 'red'}
            leftSection={connected ? <IconWifi size={16} /> : <IconWifiOff size={16} />}
          >
            {connected ? 'Vehicle Connected' : 'Vehicle Disconnected'}
          </Badge>
          <Badge 
            size="lg" 
            color={status === WebSocketStatus.OPEN ? 'green' : 'red'}
          >
            WebSocket: {status}
          </Badge>
          <Badge 
            size="lg" 
            color={subscribedChannels.includes('channels/ardulink/recv/*') ? 'green' : 'red'}
          >
            {subscribedChannels.includes('channels/ardulink/recv/*') ? 'Subscribed' : 'Not Subscribed'}
          </Badge>
        </Group>
      </Group>
      
      {/* Remove Debug cards */}

      <SimpleGrid cols={3} spacing="lg" verticalSpacing="lg">
        {/* Attitude Display Card */}
        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Card.Section withBorder inheritPadding py="xs">
            <Group justify="space-between">
              <Text fw={500}>Attitude</Text>
              <ThemeIcon size="md" color="blue" variant="light">
                <IconPlaneTilt size={16} />
              </ThemeIcon>
            </Group>
          </Card.Section>
          <Box mt="md" mb="xs">
            <Center>
              <canvas 
                ref={attitudeCanvasRef} 
                width={200} 
                height={200} 
                style={{ background: '#333', borderRadius: '50%' }}
              />
            </Center>
          </Box>
          {attitude ? (
            <SimpleGrid cols={3} spacing="xs">
              <Paper withBorder p="xs"><Text size="xs" c="dimmed">Roll</Text><Text fw={500}>{(attitude.roll * 180 / Math.PI).toFixed(1)}°</Text></Paper>
              <Paper withBorder p="xs"><Text size="xs" c="dimmed">Pitch</Text><Text fw={500}>{(attitude.pitch * 180 / Math.PI).toFixed(1)}°</Text></Paper>
              <Paper withBorder p="xs"><Text size="xs" c="dimmed">Yaw</Text><Text fw={500}>{(attitude.yaw * 180 / Math.PI).toFixed(1)}°</Text></Paper>
            </SimpleGrid>
          ) : (
             <Center h={50}><Text c="dimmed">No attitude data</Text></Center>
          )}
        </Card>

        {/* Battery Display Card */}
        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Card.Section withBorder inheritPadding py="xs">
            <Group justify="space-between">
              <Text fw={500}>Battery Status</Text>
              <ThemeIcon size="md" color="blue" variant="light"><IconBattery size={16} /></ThemeIcon>
            </Group>
          </Card.Section>
          <Box mt="md">
            {sysStatus ? (
              <>
                <Group mt="md" mb="xs">
                  <RingProgress size={80} roundCaps thickness={8} sections={[{ value: sysStatus.battery_remaining || 0, color: sysStatus.battery_remaining > 20 ? 'green' : sysStatus.battery_remaining > 10 ? 'orange' : 'red' }]} label={<Text fw={700} ta="center" size="lg">{sysStatus.battery_remaining || 0}%</Text>} />
                  <Stack gap="xs" style={{ flex: 1 }}>
                    <Group justify="space-between"><Text size="sm">Voltage:</Text><Text fw={500}>{(sysStatus.voltage_battery / 1000).toFixed(2)} V</Text></Group>
                    <Group justify="space-between"><Text size="sm">Current:</Text><Text fw={500}>{(sysStatus.current_battery / 100).toFixed(2)} A</Text></Group>
                    <Progress value={sysStatus.battery_remaining || 0} color={sysStatus.battery_remaining > 20 ? 'green' : sysStatus.battery_remaining > 10 ? 'orange' : 'red'} size="md" />
                  </Stack>
                </Group>
              </>
            ) : battery ? (
               <>
                  <Group mt="md" mb="xs">
                    <RingProgress size={80} roundCaps thickness={8} sections={[{ value: battery.battery_remaining || 0, color: battery.battery_remaining > 20 ? 'green' : battery.battery_remaining > 10 ? 'orange' : 'red' }]} label={<Text fw={700} ta="center" size="lg">{battery.battery_remaining || 0}%</Text>} />
                    <Stack gap="xs" style={{ flex: 1 }}>
                      <Group justify="space-between"><Text size="sm">Main Cell:</Text><Text fw={500}>{battery.voltages && battery.voltages.length > 0 ? (battery.voltages[0] / 1000).toFixed(2) : '?'} V</Text></Group>
                      <Group justify="space-between"><Text size="sm">Current:</Text><Text fw={500}>{battery.current_battery !== undefined ? (battery.current_battery / 100).toFixed(2) : '?'} A</Text></Group>
                      <Group justify="space-between"><Text size="sm">Temp:</Text><Text fw={500}>{battery.temperature !== undefined ? (battery.temperature / 100).toFixed(1) : '?'} °C</Text></Group>
                    </Stack>
                  </Group>
                  <Progress value={battery.battery_remaining || 0} color={battery.battery_remaining > 20 ? 'green' : battery.battery_remaining > 10 ? 'orange' : 'red'} size="md" />
               </>
            ) : (
              <Center h={150}><Text c="dimmed">No battery data</Text></Center>
            )}
          </Box>
        </Card>

        {/* Position Display Card */}
        <Card shadow="sm" padding="lg" radius="md" withBorder>
           <Card.Section withBorder inheritPadding py="xs">
             <Group justify="space-between"><Text fw={500}>Position</Text><ThemeIcon size="md" color="blue" variant="light"><IconGps size={16} /></ThemeIcon></Group>
           </Card.Section>
           <Box mt="md">
             {position ? (
               <Stack gap="md">
                 <Group grow><Paper withBorder p="xs"><Text size="xs" c="dimmed">Lat</Text><Text fw={500}>{formatLatLon(position.lat)}</Text></Paper><Paper withBorder p="xs"><Text size="xs" c="dimmed">Lon</Text><Text fw={500}>{formatLatLon(position.lon)}</Text></Paper></Group>
                 <Group grow><Paper withBorder p="xs"><Text size="xs" c="dimmed">Alt (MSL)</Text><Text fw={500}>{formatAltitude(position.alt)} m</Text></Paper><Paper withBorder p="xs"><Text size="xs" c="dimmed">Rel Alt</Text><Text fw={500}>{formatAltitude(position.relative_alt)} m</Text></Paper></Group>
                 <Group grow><Paper withBorder p="xs"><Text size="xs" c="dimmed">Speed</Text><Text fw={500}>{Math.sqrt(Math.pow(position.vx, 2) + Math.pow(position.vy, 2)) / 100} m/s</Text></Paper><Paper withBorder p="xs"><Text size="xs" c="dimmed">Heading</Text><Text fw={500}>{(position.hdg / 100).toFixed(1)}°</Text></Paper></Group>
               </Stack>
             ) : (
               <Center h={150}><Text c="dimmed">No position data</Text></Center>
             )}
           </Box>
        </Card>
        
        {/* Terrain Display Card */}
        <Card shadow="sm" padding="lg" radius="md" withBorder>
           <Card.Section withBorder inheritPadding py="xs">
             <Group justify="space-between"><Text fw={500}>Terrain</Text><ThemeIcon size="md" color="blue" variant="light"><IconGps size={16} /></ThemeIcon></Group>
           </Card.Section>
           <Box mt="md">
             {terrain ? (
               <Stack gap="md">
                 <Group grow><Paper withBorder p="xs"><Text size="xs" c="dimmed">Terrain Hgt</Text><Text fw={500}>{formatAltitude(terrain.terrain_height)} m</Text></Paper><Paper withBorder p="xs"><Text size="xs" c="dimmed">Current Hgt</Text><Text fw={500}>{formatAltitude(terrain.current_height)} m</Text></Paper></Group>
                 <Group grow><Paper withBorder p="xs"><Text size="xs" c="dimmed">Spacing</Text><Text fw={500}>{terrain.spacing} m</Text></Paper><Paper withBorder p="xs"><Text size="xs" c="dimmed">Tiles</Text><Text fw={500}>L:{terrain.loaded} P:{terrain.pending}</Text></Paper></Group>
               </Stack>
             ) : (
               <Center h={150}><Text c="dimmed">No terrain data</Text></Center>
             )}
           </Box>
        </Card>

        {/* Status Messages Card */}
        <Card shadow="sm" padding="lg" radius="md" withBorder style={{ gridColumn: 'span 1' }}> {/* Changed span */}
           <Card.Section withBorder inheritPadding py="xs">
             <Group justify="space-between"><Text fw={500}>Status</Text><ThemeIcon size="md" color="blue" variant="light"><IconMessage size={16} /></ThemeIcon></Group>
           </Card.Section>
           <ScrollArea h={200} mt="md">
             {statusText.length > 0 ? (
               <Stack gap="xs">
                 {statusText.map((msg, i) => (
                   <Paper key={i} withBorder p="xs" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                     <Group>
                       <ThemeIcon size="sm" color={getSeverityColor(msg.severity)} variant="light">{getSeverityIcon(msg.severity)}</ThemeIcon>
                       <Text size="sm" title={msg.text}>{msg.text}</Text>
                     </Group>
                   </Paper>
                 ))}
               </Stack>
             ) : (
               <Center h={150}><Text c="dimmed">No status messages</Text></Center>
             )}
           </ScrollArea>
        </Card>

        {/* System Info Card */}
        <Card shadow="sm" padding="lg" radius="md" withBorder>
           <Card.Section withBorder inheritPadding py="xs">
             <Group justify="space-between"><Text fw={500}>System</Text><ThemeIcon size="md" color="blue" variant="light"><IconHeartRateMonitor size={16} /></ThemeIcon></Group>
           </Card.Section>
           <Box mt="md">
             {heartbeat ? (
               <Stack gap="md">
                 <SimpleGrid cols={2}>
                   <Paper withBorder p="xs"><Text size="xs" c="dimmed">Type</Text><Text fw={500}>{heartbeat.mavtype === 1 ? 'Fixed Wing' : heartbeat.mavtype === 2 ? 'Quadcopter' : heartbeat.mavtype === 13 ? 'Rover' : heartbeat.mavtype === 6 ? 'Heli' : `Type ${heartbeat.mavtype}`}</Text></Paper>
                   <Paper withBorder p="xs"><Text size="xs" c="dimmed">Autopilot</Text><Text fw={500}>{heartbeat.autopilot === 3 ? 'ArduPilot' : heartbeat.autopilot === 12 ? 'PX4' : `Type ${heartbeat.autopilot}`}</Text></Paper>
                 </SimpleGrid>
                 <Paper withBorder p="xs">
                   <Text size="xs" c="dimmed">Status</Text>
                   <Badge fullWidth mt={4}>{heartbeat.system_status === 3 ? 'Standby' : heartbeat.system_status === 4 ? 'Active' : `Status ${heartbeat.system_status}`}</Badge>
                 </Paper>
                 {sysStatus && (
                   <Paper withBorder p="xs">
                     <Text size="xs" c="dimmed">CPU Load</Text>
                     <Progress value={sysStatus.load / 10} color={sysStatus.load < 500 ? 'green' : sysStatus.load < 800 ? 'yellow' : 'red'} size="md" mt={4} />
                     <Text ta="right" size="xs">{(sysStatus.load / 10).toFixed(1)}%</Text>
                   </Paper>
                 )}
               </Stack>
             ) : (
               <Center h={150}><Text c="dimmed">No system data</Text></Center>
             )}
           </Box>
        </Card>

      </SimpleGrid>
    </Container>
  );
} 