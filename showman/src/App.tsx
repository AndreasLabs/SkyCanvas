import React, { useCallback, useEffect, useRef } from 'react';

import { MantineProvider } from '@mantine/core'
import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';
import { theme } from './theme';
import { Notifications } from '@mantine/notifications';
import { BrowserRouter } from 'react-router-dom';
import { AppRoutes } from './routes';
import { useRedisWebSocket, WebSocketStatus } from './hooks/useRedisWebSocket';
import { useRedisStore } from './store/redisStore';

// Define an interface for Redis messages that's compatible with the hook type
interface RedisMessage {
  channel?: string;
  content?: string;
  message?: string;
  [key: string]: unknown;
}

// Set of message types we're primarily interested in
const PRIMARY_MESSAGE_TYPES = new Set([
  'HEARTBEAT',
  'ATTITUDE',
  'BATTERY_STATUS',
  'GLOBAL_POSITION_INT',
  'TERRAIN_REPORT',
  'STATUSTEXT',
  'SYS_STATUS'
]);

function App() {
  const { setStatus, addMessage, addSubscribedChannel } = useRedisStore();
  
  // Throttle logging - track last log time for each message type
  const lastLogTime = useRef<Record<string, number>>({});
  // Track processed message count in the last second for rate limiting
  const processedCount = useRef(0);
  const lastRateLimitReset = useRef(Date.now());

  // Memoize the callback functions with proper types
  const handleStatusChange = useCallback((newStatus: WebSocketStatus) => {
    setStatus(newStatus);
  }, [setStatus]);

  // Throttle function for logging
  const shouldLogMessage = useCallback((messageType: string) => {
    const now = Date.now();
    const lastTime = lastLogTime.current[messageType] || 0;
    
    // Log primary messages more frequently (once per second)
    const throttleInterval = PRIMARY_MESSAGE_TYPES.has(messageType) ? 1000 : 5000;
    
    if (now - lastTime > throttleInterval) {
      lastLogTime.current[messageType] = now;
      return true;
    }
    return false;
  }, []);

  // The handler needs to work with either RedisMessage or Record<string, unknown>
  const handleMessage = useCallback((data: RedisMessage | Record<string, unknown>) => {
    // Rate limit processing - reset counter every second
    const now = Date.now();
    if (now - lastRateLimitReset.current > 1000) {
      processedCount.current = 0;
      lastRateLimitReset.current = now;
    }
    
    // If we've processed too many messages in this second, throttle
    if (processedCount.current > 100) {
      return; // Skip processing if rate limit exceeded
    }
    
    // Count this message
    processedCount.current++;
    
    if (data && typeof data === 'object' && 'channel' in data) {
      const channel = data.channel as string;
      const content = 'content' in data ? data.content as string : 
                      'message' in data ? data.message as string : '';
      
      // Always add the message to the store regardless of logging
      addMessage(channel, content);
      
      // Only process and log ArduPilot messages that we care about
      if (channel.includes('ardulink')) {
        try {
          const parsedContent = JSON.parse(content);
          const messageType = parsedContent.type;
          
          // Only log if this message type hasn't been logged recently
          if (shouldLogMessage(messageType)) {
            // Only log specific message types in detail
            if (PRIMARY_MESSAGE_TYPES.has(messageType)) {
              console.log(`${messageType} message received on ${channel}`);
            }
            
            // Special handling for STATUSTEXT
            if (messageType === 'STATUSTEXT') {
              console.log('STATUSTEXT:', {
                severity: parsedContent.severity,
                text: parsedContent.text,
                isArray: Array.isArray(parsedContent.text)
              });
            }
          }
        } catch (error) {
          // Only log parsing errors occasionally
          if (shouldLogMessage('parse_error')) {
            console.log('Failed to parse message content', error);
          }
        }
      }
    }
  }, [addMessage, shouldLogMessage]);

  // Initialize the WebSocket connection with hooks
  const { status, subscribe } = useRedisWebSocket({
    onStatusChange: handleStatusChange,
    onMessage: handleMessage,
  });

  // Subscribe to required channels once connected
  useEffect(() => {
    if (status === WebSocketStatus.OPEN) {
      console.log('App: WebSocket connected - subscribing to channels');
      
      // Only subscribe to ArduPilot channels we need
      subscribe('channels/ardulink/recv/*');
      addSubscribedChannel('channels/ardulink/recv/*');
      
      console.log('App: Subscribed to ArduPilot channels');
    }
  }, [status, subscribe, addSubscribedChannel]);

  return (
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <Notifications />
      <BrowserRouter>
        <AppRoutes />
      </BrowserRouter>
    </MantineProvider>
  )
}

export default App