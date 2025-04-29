import { createTheme } from '@mantine/core';
import { THEME_COLORS } from './constants';

export const theme = createTheme({
  fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Helvetica, Arial, sans-serif',
  headings: {
    fontFamily: 'Space Grotesk, Inter, sans-serif',
    fontWeight: '500',
  },
  primaryColor: 'blue',
  components: {
    Paper: {
      defaultProps: {
        className: 'neo-glass',
      },
    },
    Card: {
      defaultProps: {
        className: 'neo-glass',
      },
    },
    Modal: {
      styles: {
        content: {
          background: `${THEME_COLORS.background.primary} !important`,
          backdropFilter: 'blur(20px) saturate(180%)',
          border: `1px solid ${THEME_COLORS.border.light}`,
          boxShadow: THEME_COLORS.shadow.primary,
        },
      },
    },
    AppShell: {
      styles: {
        main: {
          background: 'transparent',
        },
        header: {
          background: THEME_COLORS.background.primary,
          backdropFilter: 'blur(20px) saturate(180%)',
          borderBottom: `1px solid ${THEME_COLORS.border.light}`,
          boxShadow: THEME_COLORS.shadow.header,
        },
        navbar: {
          background: THEME_COLORS.background.primary,
          backdropFilter: 'blur(20px) saturate(180%)',
          borderRight: `1px solid ${THEME_COLORS.border.light}`,
          boxShadow: THEME_COLORS.shadow.navbar,
        },
      },
    },
    Button: {
      defaultProps: {
        variant: 'filled',
      },
      styles: {
        root: {
          background: THEME_COLORS.background.secondary,
          backdropFilter: 'blur(20px)',
          border: `1px solid ${THEME_COLORS.border.light}`,
          color: THEME_COLORS.text.primary,
          '&:hover': {
            background: THEME_COLORS.background.hover,
            borderColor: THEME_COLORS.border.glow,
            boxShadow: THEME_COLORS.shadow.glow,
          },
        },
      },
    },
    Title: {
      styles: {
        root: {
          color: THEME_COLORS.text.primary,
          letterSpacing: '-0.5px',
        },
      },
    },
    Text: {
      styles: {
        root: {
          color: THEME_COLORS.text.secondary,
        },
      },
    },
    NavLink: {
      styles: {
        root: {
          '&[data-active]': {
            backgroundColor: 'rgba(51, 154, 240, 0.15) !important',
            borderLeft: '2px solid #339af0 !important',
          },
        },
      },
    },
    Menu: {
      styles: {
        dropdown: {
          background: THEME_COLORS.background.primary,
          backdropFilter: 'blur(20px) saturate(180%)',
          border: `1px solid ${THEME_COLORS.border.light}`,
          boxShadow: THEME_COLORS.shadow.primary,
        },
        item: {
          '&[data-hovered]': {
            background: THEME_COLORS.background.hover,
          },
        },
      },
    },
  },
}); 