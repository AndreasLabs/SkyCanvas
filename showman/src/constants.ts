// API Configuration
export const API_URL = 'http://localhost:4151';

// Query Configuration
export const QUERY_CONFIG = {
  staleTime: 1000 * 60 * 5, // 5 minutes
  gcTime: 1000 * 60 * 30, // 30 minutes
  retry: 1,
  refetchOnWindowFocus: false,
  refetchOnMount: 'always',
} as const;

// Theme Configuration
export const THEME_COLORS = {
  background: {
    primary: 'rgba(13, 15, 15, 0.97)',
    secondary: 'rgba(26, 22, 13, 0.97)',
    hover: 'rgba(32, 28, 15, 0.97)',
  },
  border: {
    light: 'rgba(255, 255, 255, 0.06)',
    glow: 'rgba(51, 154, 240, 0.5)',
  },
  shadow: {
    primary: '0 8px 32px rgba(0, 0, 0, 0.8)',
    header: '0 4px 32px rgba(0, 0, 0, 0.8)',
    navbar: '4px 0 32px rgba(0, 0, 0, 0.8)',
    glow: '0 0 20px rgba(51, 154, 240, 0.2)',
  },
  text: {
    primary: '#ffffff',
    secondary: 'rgba(255, 255, 255, 0.75)',
    accent: '#339af0',
  },
} as const;

// Layout Configuration
export const LAYOUT = {
  header: {
    height: 60,
  },
  navbar: {
    width: 300,
    breakpoint: 'sm',
  },
} as const; 