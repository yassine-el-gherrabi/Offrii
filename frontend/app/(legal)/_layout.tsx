import { Platform } from 'react-native';
import { Stack, router } from 'expo-router';
import { useTranslation } from 'react-i18next';
import { IconButton } from 'react-native-paper';
import { colors } from '@/src/theme';

export default function LegalLayout() {
  const { t } = useTranslation();

  return (
    <Stack
      screenOptions={{
        headerShown: true,
        headerTintColor: colors.primary,
        headerStyle: { backgroundColor: colors.background },
        contentStyle: { backgroundColor: colors.background },
        headerLeft: () => (
          <IconButton
            icon={Platform.OS === 'ios' ? 'chevron-left' : 'arrow-left'}
            iconColor={colors.primary}
            onPress={() => router.back()}
          />
        ),
      }}
    >
      <Stack.Screen
        name="legal-notice"
        options={{ title: t('legal.legalNotice.title') }}
      />
      <Stack.Screen
        name="privacy-policy"
        options={{ title: t('legal.privacyPolicy.title') }}
      />
    </Stack>
  );
}
