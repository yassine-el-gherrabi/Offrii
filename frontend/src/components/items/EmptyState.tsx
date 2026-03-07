import { View, StyleSheet } from 'react-native';
import { Text } from 'react-native-paper';
import { MaterialCommunityIcons } from '@expo/vector-icons';
import { useTranslation } from 'react-i18next';

import { colors, spacing } from '@/src/theme';
import type { ItemStatus } from '@/src/types/items';

interface EmptyStateProps {
  status: ItemStatus;
}

export function EmptyState({ status }: EmptyStateProps) {
  const { t } = useTranslation();

  const icon = status === 'active' ? 'gift-outline' : 'check-circle-outline';
  const title =
    status === 'active' ? t('list.empty.activeTitle') : t('list.empty.purchasedTitle');
  const subtitle = status === 'active' ? t('list.empty.activeSubtitle') : undefined;

  return (
    <View style={styles.container} testID="empty-state">
      <MaterialCommunityIcons name={icon} size={64} color={colors.textSecondary} />
      <Text variant="titleMedium" style={styles.title}>
        {title}
      </Text>
      {subtitle && (
        <Text variant="bodyMedium" style={styles.subtitle}>
          {subtitle}
        </Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: spacing.xl * 2,
  },
  title: {
    marginTop: spacing.md,
    color: colors.textSecondary,
  },
  subtitle: {
    marginTop: spacing.sm,
    color: colors.textSecondary,
  },
});
