import { StyleSheet } from 'react-native';
import { colors, spacing, borderRadius } from '@/src/theme';

export const authStyles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  flex: {
    flex: 1,
  },
  scroll: {
    flexGrow: 1,
    justifyContent: 'center',
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.xl,
  },
  logoContainer: {
    alignItems: 'center',
    marginBottom: spacing.md,
  },
  title: {
    textAlign: 'center',
    fontWeight: '700',
    color: colors.text,
    marginBottom: spacing.lg,
  },
  errorBanner: {
    backgroundColor: '#FEE2E2',
    borderRadius: borderRadius.sm,
    padding: spacing.md,
    marginBottom: spacing.md,
  },
  errorBannerText: {
    color: colors.error,
    textAlign: 'center',
    fontSize: 14,
  },
  input: {
    backgroundColor: colors.inputBackground,
  },
  inputOutline: {
    borderRadius: borderRadius.sm,
  },
  button: {
    marginTop: spacing.sm,
    borderRadius: borderRadius.sm,
    backgroundColor: colors.primary,
  },
  buttonContent: {
    paddingVertical: spacing.sm,
  },
  buttonLabel: {
    fontSize: 16,
    fontWeight: '600',
  },
  linkRow: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginTop: spacing.lg,
  },
  linkText: {
    color: colors.textSecondary,
  },
  link: {
    color: colors.primary,
    fontWeight: '600',
  },
});
