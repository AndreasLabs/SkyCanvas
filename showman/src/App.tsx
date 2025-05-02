import React, { useCallback } from 'react';

import { MantineProvider } from '@mantine/core'
import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';
import { theme } from './theme';
import { Notifications } from '@mantine/notifications';
import { BrowserRouter } from 'react-router-dom';
import { AppRoutes } from './routes';
import { useRedisWebSocket, WebSocketStatus } from './hooks/useRedisWebSocket';
import { useRedisStore } from './store/redisStore';

function App() {
  const { setStatus, addMessage } = useRedisStore();

  // Memoize the callback functions with proper types
  const handleStatusChange = useCallback((newStatus: WebSocketStatus) => {
    setStatus(newStatus);
  }, [setStatus]);

  const handleMessage = useCallback((data: any) => {
    if (data && typeof data === 'object' && 'channel' in data) {
      const channel = data.channel as string;
      const content = 'content' in data ? data.content as string : 
                      'message' in data ? data.message as string : '';
      addMessage(channel, content);
    }
  }, [addMessage]);

  // Initialize the WebSocket connection here
  useRedisWebSocket({
    onStatusChange: handleStatusChange,
    onMessage: handleMessage,
  });

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
