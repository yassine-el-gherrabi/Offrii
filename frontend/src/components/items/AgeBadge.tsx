import { View, StyleSheet } from 'react-native';
import { Text } from 'react-native-paper';
import { useTranslation } from 'react-i18next';

import { daysAgo, ageColor, formatRelativeDate } from '@/src/utils/dates';
import { spacing, borderRadius } from '@/src/theme';

interface AgeBadgeProps {
  createdAt: string;
  size?: 'small' | 'medium';
}

export function AgeBadge({ createdAt, size = 'small' }: AgeBadgeProps) {
  const { t } = useTranslation();
  const days = daysAgo(createdAt);
  const color = ageColor(days);

  return (
    <View testID="age-badge" style={styles.container}>
      <View style={[styles.badge, { backgroundColor: color }]}>
        <Text style={styles.badgeText} testID="age-badge-text">
          {formatRelativeDate(createdAt)}
        </Text>
      </View>
      {size === 'medium' && (
        <Text variant="bodySmall" style={[styles.subtitle, { color }]} testID="age-badge-subtitle">
          {t('detail.since', { count: days })}
        </Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
  },
  badge: {
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: borderRadius.sm,
  },
  badgeText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '600',
  },
  subtitle: {
    marginTop: spacing.xs,
    fontSize: 12,
  },
});
