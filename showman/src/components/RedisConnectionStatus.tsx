import React from 'react';
import { Badge, Group, Text } from '@mantine/core';
import { IconPlugConnected, IconPlugConnectedX } from '@tabler/icons-react';
import { useRedisStore } from '../store/redisStore';
import { WebSocketStatus } from '../hooks/useRedisWebSocket';

export function RedisConnectionStatus() {
  const { status } = useRedisStore();

  return (
    <Group gap="xs">
      <Badge 
        color={status === WebSocketStatus.OPEN ? 'green' : 'red'}
        leftSection={
          status === WebSocketStatus.OPEN 
            ? <IconPlugConnected size={14} />
            : <IconPlugConnectedX size={14} />
        }
      >
        {status === WebSocketStatus.OPEN ? 'Connected' : 'Disconnected'}
      </Badge>
      {status !== WebSocketStatus.OPEN && (
        <Text size="xs" c="dimmed">Redis WebSocket</Text>
      )}
    </Group>
  );
} 