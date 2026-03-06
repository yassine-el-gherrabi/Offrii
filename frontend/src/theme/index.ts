import { MD3LightTheme } from 'react-native-paper';

export const colors = {
  primary: '#FF6B6B',      // Corail Generous
  secondary: '#6C5CE7',    // Violet Surprise
  accent: '#FFC312',       // Or Celebration
  background: '#FFF5F5',   // Rose Murmure
  surface: '#FFFFFF',
  inputBackground: '#FFFFFF',
  inputBorder: '#E5E7EB',
  text: '#1F2937',
  textSecondary: '#6B7280',
  error: '#EF4444',
  success: '#10B981',
  // Age badges
  ageFresh: '#10B981',
  ageModerate: '#F59E0B',
  ageOld: '#EF4444',
} as const;

export const spacing = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
} as const;

export const borderRadius = {
  sm: 12,
  md: 16,
  lg: 28,
  full: 9999,
} as const;

export const typography = {
  sizes: {
    caption: 12,
    body: 14,
    subtitle: 16,
    title: 20,
    headline: 24,
  },
  weights: {
    regular: '400' as const,
    medium: '500' as const,
    semibold: '600' as const,
    bold: '700' as const,
  },
} as const;

export const paperTheme = {
  ...MD3LightTheme,
  colors: {
    ...MD3LightTheme.colors,
    primary: colors.primary,
    secondary: colors.secondary,
    background: colors.background,
    surface: colors.surface,
    error: colors.error,
    onPrimary: '#FFFFFF',
    onSecondary: '#FFFFFF',
    onBackground: colors.text,
    onSurface: colors.text,
  },
};
