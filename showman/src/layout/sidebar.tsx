import React from 'react';
import { NavLink, Stack, Text, Group } from '@mantine/core';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  IconHome,
  IconLogs,
  IconDatabase,
} from '@tabler/icons-react';

interface NavItemProps {
  label: string;
  icon: React.ReactNode;
  path: string;
  active: boolean;
  disabled?: boolean;
  selectedItem?: string;
  onClick: () => void;
}

function NavItem({ label, icon, active, disabled, selectedItem, onClick }: NavItemProps) {
  return (
    <NavLink
      label={
        <Group gap="xs">
          {icon}
          <Stack gap={0}>
            <Text size="sm">{label}</Text>
            {selectedItem && (
              <Text size="xs" c="dimmed" truncate>
                {selectedItem}
              </Text>
            )}
          </Stack>
        </Group>
      }
      data-active={active || undefined}
      disabled={disabled}
      onClick={onClick}
    />
  );
}

export function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();

  const navItems = [
    {
      label: 'Home',
      icon: <IconHome size="1.2rem" />,
      path: '/',
      onClick: () => navigate('/'),
    },
    {
      label: 'Redis Demo',
      icon: <IconDatabase size="1.2rem" />,
      path: '/redis-demo',
      onClick: () => navigate('/redis-demo'),
    },
    {
      label: 'Log Entry',
      icon: <IconLogs size="1.2rem" />,
      path: '/logs',
      onClick: () => navigate('/logs'),
    },
  ];

  return (
    <Stack p="md">
      {navItems.map((item) => (
        <NavItem 
          key={item.label} 
          {...item} 
          active={location.pathname === item.path}
        />
      ))}
    </Stack>
  );
} 