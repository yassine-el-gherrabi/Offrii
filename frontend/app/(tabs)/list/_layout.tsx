import { Stack } from 'expo-router';
import { useTranslation } from 'react-i18next';

export default function ListLayout() {
  const { t } = useTranslation();

  return (
    <Stack screenOptions={{ headerShown: false }}>
      <Stack.Screen name="index" />
      <Stack.Screen
        name="[id]"
        options={{
          headerShown: true,
          title: t('detail.title'),
          headerBackTitle: t('list.backTitle'),
        }}
      />
    </Stack>
  );
}
