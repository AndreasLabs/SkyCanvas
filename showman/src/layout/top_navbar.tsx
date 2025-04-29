import React from 'react';
import { 
  Group, 
  Container,
  Flex,
  AppShell,
  Text,
  UnstyledButton,
  Burger,
} from '@mantine/core';
import { IconRocket } from '@tabler/icons-react';
import { LAYOUT } from '../constants';
import { useNavigate } from 'react-router-dom';

interface TopNavbarProps {
  navbarOpened: boolean;
  onNavbarToggle: () => void;
}

export function TopNavbar({ navbarOpened, onNavbarToggle }: TopNavbarProps) {
  const navigate = useNavigate();

  return (
    <AppShell.Header h={LAYOUT.header.height}>
      <Container fluid h="100%">
        <Flex h="100%" align="center" justify="space-between">
          <Group>
            <Burger opened={navbarOpened} onClick={onNavbarToggle} size="sm" />
            <UnstyledButton onClick={() => navigate('/')}>
              <Group gap="xs">
                <IconRocket size={30} />
                <Text size="lg" fw={700}>Cursed Web Template</Text>
              </Group>
            </UnstyledButton>
          </Group>
        </Flex>
      </Container>
    </AppShell.Header>
  );
} 
