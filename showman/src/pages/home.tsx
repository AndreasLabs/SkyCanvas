import { useState } from 'react';
import { 
  Container, 
  Grid, 
  Card, 
  Text, 
  Title, 
  Button, 
  Group, 
  Stack, 
  Badge, 
  Progress, 
  ActionIcon, 
  Flex,
  SimpleGrid,
  Paper,
  RingProgress
} from '@mantine/core';
import { 
  IconDrone, 
  IconMap2, 
  IconCalendarEvent, 
  IconChartLine, 
  IconPlus, 
  IconRefresh,
  IconBattery3,
  IconCloudUpload,
  IconRoute,
  IconAlertCircle
} from '@tabler/icons-react';

export function Home() {
  const [isLoading, setIsLoading] = useState(false);
  
  const refreshData = () => {
    setIsLoading(true);
    setTimeout(() => setIsLoading(false), 1500);
  };

  return (
    <Container size="xl" py="xl">
      {/* Welcome Section */}
      <Card className="welcome-card home-animate" p="xl" radius="md" mb="xl">
        <Grid>
          <Grid.Col span={{ base: 12, md: 8 }}>
            <Title order={2} mb="xs">Welcome to Drone Planner Dashboard</Title>
            <Text c="dimmed" mb="lg">
              Plan, monitor, and analyze your drone operations from a single interface.
            </Text>
            <Group>
              <Button leftSection={<IconPlus size={16} />}>New Mission</Button>
              <Button variant="light" leftSection={<IconMap2 size={16} />}>View Map</Button>
            </Group>
          </Grid.Col>
          <Grid.Col span={{ base: 12, md: 4 }} style={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
            <ActionIcon 
              variant="transparent" 
              size={120} 
              className="glass-icon glow-effect"
            >
              <IconDrone size={60} />
            </ActionIcon>
          </Grid.Col>
        </Grid>
      </Card>

      {/* Stats Overview */}
      <SimpleGrid cols={{ base: 1, sm: 2, md: 4 }} mb="xl" className="home-animate home-animate-delay-1">
        {[
          { title: 'Active Drones', value: '4/6', icon: <IconDrone size={24} />, color: 'blue' },
          { title: 'Planned Missions', value: '12', icon: <IconRoute size={24} />, color: 'teal' },
          { title: 'Data Collected', value: '1.4 TB', icon: <IconCloudUpload size={24} />, color: 'violet' },
          { title: 'Battery Status', value: '76%', icon: <IconBattery3 size={24} />, color: 'yellow' }
        ].map((stat, index) => (
          <Card key={index} className="neo-glass" p="md" radius="md">
            <Group justify="space-between" mb="xs">
              <Text size="sm" fw={500} c="dimmed">{stat.title}</Text>
              <ActionIcon variant="subtle" color={stat.color} className={isLoading ? 'rotating' : ''}>
                {stat.icon}
              </ActionIcon>
            </Group>
            <Text fw={700} size="xl">{stat.value}</Text>
          </Card>
        ))}
      </SimpleGrid>

      {/* Main Content */}
      <Grid gutter="xl" className="home-animate home-animate-delay-2">
        {/* Upcoming Missions */}
        <Grid.Col span={{ base: 12, md: 8 }}>
          <Card className="neo-glass" p="md" radius="md">
            <Group justify="space-between" mb="md">
              <Title order={3}>Upcoming Missions</Title>
              <Button 
                variant="subtle" 
                leftSection={<IconRefresh size={16} className={isLoading ? 'rotating' : ''} />}
                onClick={refreshData}
                loading={isLoading}
              >
                Refresh
              </Button>
            </Group>
            
            <Stack>
              {[
                {
                  id: 1,
                  name: 'Area Survey - North Field',
                  date: '2024-03-15',
                  status: 'scheduled',
                  progress: 0
                },
                {
                  id: 2,
                  name: 'Crop Monitoring - South Section',
                  date: '2024-03-16',
                  status: 'in-progress',
                  progress: 45
                },
                {
                  id: 3,
                  name: 'Infrastructure Inspection',
                  date: '2024-03-17',
                  status: 'pending',
                  progress: 0
                }
              ].map((mission) => (
                <Paper key={mission.id} p="md" radius="md" withBorder>
                  <Group justify="space-between">
                    <Stack gap={0}>
                      <Text fw={500}>{mission.name}</Text>
                      <Text size="sm" c="dimmed">{mission.date}</Text>
                    </Stack>
                    <Badge color={
                      mission.status === 'scheduled' ? 'blue' :
                      mission.status === 'in-progress' ? 'yellow' :
                      'gray'
                    }>
                      {mission.status}
                    </Badge>
                  </Group>
                  <Progress value={mission.progress} mt="md" size="sm" />
                </Paper>
              ))}
            </Stack>
          </Card>
        </Grid.Col>

        {/* System Status */}
        <Grid.Col span={{ base: 12, md: 4 }}>
          <Stack>
            <Card className="neo-glass" p="md" radius="md">
              <Title order={3} mb="md">System Status</Title>
              <Stack gap="md">
                <Group justify="space-between">
                  <Text>Connection Status</Text>
                  <Badge color="green">Connected</Badge>
                </Group>
                <Group justify="space-between">
                  <Text>Last Update</Text>
                  <Text size="sm" c="dimmed">2 minutes ago</Text>
                </Group>
                <Group justify="space-between">
                  <Text>System Health</Text>
                  <RingProgress
                    size={80}
                    thickness={8}
                    roundCaps
                    sections={[{ value: 85, color: 'green' }]}
                    label={
                      <Text ta="center" size="sm" fw={700}>
                        85%
                      </Text>
                    }
                  />
                </Group>
              </Stack>
            </Card>

            <Card className="neo-glass" p="md" radius="md">
              <Title order={3} mb="md">Recent Alerts</Title>
              <Stack gap="md">
                <Group>
                  <ActionIcon color="red" variant="light">
                    <IconAlertCircle size={16} />
                  </ActionIcon>
                  <Text size="sm">Low battery warning on Drone #2</Text>
                </Group>
                <Group>
                  <ActionIcon color="yellow" variant="light">
                    <IconAlertCircle size={16} />
                  </ActionIcon>
                  <Text size="sm">Weather conditions may affect flight</Text>
                </Group>
              </Stack>
            </Card>
          </Stack>
        </Grid.Col>
      </Grid>
    </Container>
  );
}

