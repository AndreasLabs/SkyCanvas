import { useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'

import { MantineProvider } from '@mantine/core'
import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';
import { theme } from './theme';
import { Notifications } from '@mantine/notifications';
import { BrowserRouter } from 'react-router-dom';
import { AppRoutes } from './routes';

function App() {
  const [count, setCount] = useState(0)

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
