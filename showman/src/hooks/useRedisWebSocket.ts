import { useEffect, useRef, useState, useCallback } from 'react';
import { WS_URL } from '../constants';

export enum WebSocketStatus {
  CONNECTING = 'connecting',
  OPEN = 'open',
  CLOSING = 'closing',
  CLOSED = 'closed',
}

type RedisMessage = {
  channel: string;
  message: string;
};

// Define a more specific type for the message parameter
interface UseRedisWebSocketProps {
  onMessage?: (message: RedisMessage | Record<string, unknown>) => void;
  onStatusChange?: (status: WebSocketStatus) => void;
  autoReconnect?: boolean;
  reconnectInterval?: number;
}

// Global socket - shared across all components
let socket: WebSocket | null = null;
let isInitialized = false;

export function useRedisWebSocket({
  onMessage,
  onStatusChange,
  autoReconnect = true,
  reconnectInterval = 3000,
}: UseRedisWebSocketProps = {}) {
  const [status, setStatus] = useState<WebSocketStatus>(WebSocketStatus.CLOSED);
  const [messages, setMessages] = useState<RedisMessage[]>([]);
  const reconnectTimer = useRef<number | null>(null);

  // Update local status and notify callback
  const updateStatus = useCallback((newStatus: WebSocketStatus) => {
    setStatus(newStatus);
    if (onStatusChange) {
      onStatusChange(newStatus);
    }
  }, [onStatusChange]);

  // Connect or get existing connection
  const connect = useCallback(() => {
    // If already connected or connecting, do nothing
    if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING)) {
      updateStatus(socket.readyState === WebSocket.OPEN ? WebSocketStatus.OPEN : WebSocketStatus.CONNECTING);
      return;
    }
    
    // Close existing socket if needed
    if (socket) {
      socket.close();
    }
    
    // Create new socket
    socket = new WebSocket(WS_URL);
    updateStatus(WebSocketStatus.CONNECTING);
    
    // Socket events
    socket.onopen = () => {
      updateStatus(WebSocketStatus.OPEN);
    };
    
    socket.onclose = () => {
      updateStatus(WebSocketStatus.CLOSED);
      socket = null;
      
      if (autoReconnect && reconnectTimer.current === null) {
        reconnectTimer.current = window.setTimeout(() => {
          reconnectTimer.current = null;
          connect();
        }, reconnectInterval);
      }
    };
    
    socket.onerror = () => {
      // Let onclose handle reconnection
    };
  }, [autoReconnect, reconnectInterval, updateStatus]);

  // Subscribe to a Redis channel
  const subscribe = useCallback((channel: string) => {
    if (!socket || socket.readyState !== WebSocket.OPEN) return;
    
    try {
      socket.send(JSON.stringify({
        RedisSubscribe: channel
      }));
    } catch (error) {
      console.error('Error subscribing:', error);
    }
  }, []);

  // Publish to a Redis channel
  const publish = useCallback((channel: string, message: string) => {
    if (!socket || socket.readyState !== WebSocket.OPEN) return;
    
    try {
      socket.send(JSON.stringify({
        RedisPublish: [channel, message]
      }));
    } catch (error) {
      console.error('Error publishing:', error);
    }
  }, []);

  // Set a Redis key
  const setKey = useCallback((key: string, value: string) => {
    if (!socket || socket.readyState !== WebSocket.OPEN) return;
    
    try {
      socket.send(JSON.stringify({
        RedisUpdate: [key, value]
      }));
    } catch (error) {
      console.error('Error setting key:', error);
    }
  }, []);

  // Setup and cleanup
  useEffect(() => {
    // Function to handle incoming messages
    const handleMessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data);
        setMessages(prev => [...prev, data]);
        if (onMessage) {
          onMessage(data);
        }
      } catch (error) {
        console.error('Error parsing message:', error);
      }
    };
    
    // Setup
    if (!isInitialized) {
      isInitialized = true;
      connect();
    }
    
    // Handle state of existing socket
    if (socket) {
      if (socket.readyState === WebSocket.OPEN) {
        updateStatus(WebSocketStatus.OPEN);
      } else if (socket.readyState === WebSocket.CONNECTING) {
        updateStatus(WebSocketStatus.CONNECTING);
      }
      
      // Add message listener
      socket.addEventListener('message', handleMessage);
    } else {
      connect();
    }
    
    // Cleanup
    return () => {
      if (socket) {
        socket.removeEventListener('message', handleMessage);
      }
      
      if (reconnectTimer.current !== null) {
        window.clearTimeout(reconnectTimer.current);
        reconnectTimer.current = null;
      }
    };
  }, [connect, onMessage, updateStatus]);

  return {
    status,
    messages,
    subscribe,
    publish,
    setKey,
    connect,
  };
} 