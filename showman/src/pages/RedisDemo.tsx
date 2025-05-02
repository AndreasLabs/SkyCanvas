import React, { useState, useCallback, useEffect } from 'react';
import { 
  Container, 
  Title, 
  Text, 
  TextInput, 
  Button, 
  Card, 
  Group, 
  Stack, 
  Paper,
  ActionIcon,
  Badge,
  Divider,
  Accordion,
  Menu,
  Tabs,
  ScrollArea,
  Timeline,
  ThemeIcon,
  useMantineTheme
} from '@mantine/core';
import { useRedisStore } from '../store/redisStore';
import { 
  IconTrash, 
  IconDots,
  IconPlus,
  IconFilter,
  IconMessage,
  IconAntenna,
  IconDatabase,
  IconWifi,
  IconWifiOff
} from '@tabler/icons-react';
import { useRedisWebSocket, WebSocketStatus } from '../hooks/useRedisWebSocket';

export function RedisDemo() {
  const [newChannel, setNewChannel] = useState('');
  const theme = useMantineTheme();
  
  const { 
    subscribedChannels, 
    messages, 
    status, 
    addSubscribedChannel, 
    removeSubscribedChannel,
    clearMessages
  } = useRedisStore();
  
  const { subscribe } = useRedisWebSocket();

  // Subscribe to wildcard pattern automatically when component loads
  useEffect(() => {
    if (status === WebSocketStatus.OPEN && !subscribedChannels.includes('*')) {
      subscribe('*');
      addSubscribedChannel('*');
    }
  }, [status, subscribe, addSubscribedChannel, subscribedChannels]);

  const handleSubscribe = useCallback(() => {
    if (!newChannel.trim() || status !== WebSocketStatus.OPEN) return;
    subscribe(newChannel);
    addSubscribedChannel(newChannel);
    setNewChannel('');
  }, [newChannel, subscribe, addSubscribedChannel, status]);

  const handleClearMessages = useCallback((channel?: string) => {
    clearMessages(channel);
  }, [clearMessages]);

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString();
  };
  
  // Get all messages across all channels
  const allMessages = Object.entries(messages)
    .flatMap(([channelName, channelMsgs]) => 
      channelMsgs.map((msg) => ({
        channelName,
        content: msg.content,
        timestamp: msg.timestamp,
      }))
    )
    .sort((a, b) => b.timestamp - a.timestamp) // Sort by timestamp (newest first)
    .slice(0, 100); // Limit to last 100 messages

  return (
    <Container size="lg" py="xl">
      <Card p="lg" radius="md" withBorder mb="xl">
        <Group justify="space-between">
          <Group>
            <ThemeIcon size="xl" radius="xl" variant="light" color={status === WebSocketStatus.OPEN ? 'green' : 'red'}>
              {status === WebSocketStatus.OPEN ? <IconWifi size={24} /> : <IconWifiOff size={24} />}
            </ThemeIcon>
            <div>
              <Title order={2}>Redis PubSub Monitor</Title>
              <Text c="dimmed">
                Real-time monitoring of Redis channels via WebSockets
              </Text>
            </div>
          </Group>
          <Badge 
            size="lg" 
            variant="outline" 
            color={status === WebSocketStatus.OPEN ? 'green' : 'red'}
          >
            {status === WebSocketStatus.OPEN ? 'Connected' : 'Disconnected'}
          </Badge>
        </Group>
      </Card>
      
      <Tabs defaultValue="all-messages">
        <Tabs.List>
          <Tabs.Tab 
            value="all-messages" 
            leftSection={<IconDatabase size={16} />}
          >
            All Messages
          </Tabs.Tab>
          <Tabs.Tab 
            value="channels" 
            leftSection={<IconAntenna size={16} />}
          >
            Channel Management
          </Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="all-messages" pt="md">
          <Card p="md" radius="md" withBorder>
            <Group justify="space-between" mb="md">
              <Title order={3}>Message Stream</Title>
              <Group>
                <Text c="dimmed">{allMessages.length} messages</Text>
                <ActionIcon color="red" onClick={() => handleClearMessages()} title="Clear all messages">
                  <IconTrash size={16} />
                </ActionIcon>
              </Group>
            </Group>
            
            <Divider mb="md" />
            
            <ScrollArea h={500} offsetScrollbars scrollbarSize={6}>
              {allMessages.length > 0 ? (
                <Timeline active={allMessages.length - 1} bulletSize={24} lineWidth={2}>
                  {allMessages.map((msg, index) => (
                    <Timeline.Item 
                      key={index} 
                      bullet={<IconMessage size={12} />}
                      title={
                        <Group gap="xs">
                          <Badge size="sm" color={theme.primaryColor}>{msg.channelName}</Badge>
                          <Text size="xs" c="dimmed">{formatTimestamp(msg.timestamp)}</Text>
                        </Group>
                      }
                    >
                      <Text size="sm" mt={4}>{msg.content}</Text>
                    </Timeline.Item>
                  ))}
                </Timeline>
              ) : (
                <Text ta="center" c="dimmed" py="xl">No messages received yet. Waiting for Redis activity...</Text>
              )}
            </ScrollArea>
          </Card>
        </Tabs.Panel>
        
        <Tabs.Panel value="channels" pt="md">
          <Card p="md" radius="md" withBorder>
            <Title order={3} mb="md">Channel Subscriptions</Title>
            
            <Group mb="lg">
              <TextInput
                placeholder="Channel name or pattern (e.g. 'user.*')"
                description="Use * as a wildcard to match multiple channels"
                value={newChannel}
                onChange={(e) => setNewChannel(e.target.value)}
                style={{ flex: 1 }}
                disabled={status !== WebSocketStatus.OPEN}
              />
              <Button 
                onClick={handleSubscribe}
                leftSection={<IconPlus size={14} />}
                disabled={status !== WebSocketStatus.OPEN || !newChannel.trim()}
              >
                Subscribe
              </Button>
            </Group>
            
            <Divider my="md" label="Active Subscriptions" labelPosition="center" />
            
            {subscribedChannels.length === 0 ? (
              <Text c="dimmed" ta="center" py="xl">No active subscriptions</Text>
            ) : (
              <Stack gap="sm">
                {subscribedChannels.map((channel) => (
                  <Paper key={channel} p="md" withBorder>
                    <Group justify="space-between">
                      <Group>
                        <Badge 
                          size="lg" 
                          variant={channel === '*' ? 'filled' : 'outline'}
                        >
                          {channel}
                        </Badge>
                        {channel === '*' && (
                          <Text size="xs" c="dimmed">All channels</Text>
                        )}
                      </Group>
                      
                      <Menu shadow="md" position="bottom-end">
                        <Menu.Target>
                          <ActionIcon>
                            <IconDots size={16} />
                          </ActionIcon>
                        </Menu.Target>
                        
                        <Menu.Dropdown>
                          <Menu.Label>Channel options</Menu.Label>
                          <Menu.Item 
                            onClick={() => handleClearMessages(channel)}
                            leftSection={<IconTrash size={14} />}
                            color="red"
                          >
                            Clear messages
                          </Menu.Item>
                          {channel !== '*' && (
                            <Menu.Item 
                              onClick={() => removeSubscribedChannel(channel)}
                              leftSection={<IconFilter size={14} />}
                              color="red"
                            >
                              Unsubscribe
                            </Menu.Item>
                          )}
                        </Menu.Dropdown>
                      </Menu>
                    </Group>
                    
                    <Group mt="xs">
                      <Text size="sm">
                        Messages: <b>{messages[channel]?.length || 0}</b>
                      </Text>
                    </Group>
                  </Paper>
                ))}
              </Stack>
            )}
          </Card>
        </Tabs.Panel>
      </Tabs>
      
      <Card p="lg" radius="md" withBorder mt="xl">
        <Accordion>
          <Accordion.Item value="about">
            <Accordion.Control>
              <Title order={4}>About Redis PubSub</Title>
            </Accordion.Control>
            <Accordion.Panel>
              <Text>
                Redis Pub/Sub is a messaging paradigm where senders (publishers) send messages to channels without 
                knowledge of specific receivers (subscribers). Subscribers receive messages by expressing interest in 
                specific channels.
              </Text>
              <Text mt="md">
                This demo establishes a WebSocket connection to our Redis bridge. The bridge converts WebSocket 
                messages to Redis PUB/SUB operations and vice versa, allowing real-time monitoring of Redis activity.
              </Text>
              <Text mt="md" fw={500}>
                Pattern subscriptions using wildcards (like "*") are fully supported. When using a pattern like "*", 
                you'll receive messages from all channels that match the pattern.
              </Text>
              <Text mt="md">
                This pattern is ideal for building real-time features like notifications, chat systems, or 
                monitoring dashboards.
              </Text>
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion>
      </Card>
    </Container>
  );
} 