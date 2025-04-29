import React, { useState } from 'react';
import { AppShell } from '@mantine/core';
import { TopNavbar } from './top_navbar';
import { Sidebar } from './sidebar';
import { Outlet } from 'react-router-dom';
import { LAYOUT } from '../constants';

export function AppLayout() {
  const [opened, setOpened] = useState(true);

  return (
    <AppShell
      header={{ height: LAYOUT.header.height }}
      navbar={{
        width: LAYOUT.navbar.width,
        breakpoint: LAYOUT.navbar.breakpoint,
        collapsed: { desktop: !opened, mobile: !opened }
      }}
      padding="md"
      styles={{
        header: {
          zIndex: 200,
        },
        navbar: {
          zIndex: 100,
        }
      }}
    >
      <AppShell.Header>
        <TopNavbar navbarOpened={opened} onNavbarToggle={() => setOpened(!opened)} />
      </AppShell.Header>

      <AppShell.Navbar>
        <Sidebar />
      </AppShell.Navbar>

      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
} 