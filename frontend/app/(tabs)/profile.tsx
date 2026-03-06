import { View, StyleSheet } from 'react-native';
import { Text, Button } from 'react-native-paper';
import { useTranslation } from 'react-i18next';
import { router } from 'expo-router';

import { useAuthStore } from '@/src/stores/auth';
import { colors, spacing, borderRadius } from '@/src/theme';
import { ROUTES } from '@/src/constants/routes';

export default function ProfileScreen() {
  const { t } = useTranslation();
  const logout = useAuthStore((s) => s.logout);

  async function handleLogout() {
    await logout();
    router.replace(ROUTES.LOGIN);
  }

  return (
    <View style={styles.container}>
      <Text variant="headlineMedium">{t('profile.title')}</Text>

      <Button
        mode="outlined"
        onPress={handleLogout}
        textColor={colors.error}
        style={styles.logoutButton}
        testID="logout-button"
      >
        {t('profile.logout')}
      </Button>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: spacing.lg,
  },
  logoutButton: {
    marginTop: spacing.xl,
    borderColor: colors.error,
    borderRadius: borderRadius.sm,
  },
});
