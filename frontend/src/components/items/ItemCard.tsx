import { StyleSheet, Pressable, View } from 'react-native';
import { Surface, Text } from 'react-native-paper';

import { colors, spacing, borderRadius } from '@/src/theme';
import { PriorityIndicator } from './PriorityIndicator';
import { AgeBadge } from './AgeBadge';
import { CategoryChip } from './CategoryChip';
import type { ItemResponse, CategoryResponse } from '@/src/types/items';

interface ItemCardProps {
  item: ItemResponse;
  categories: CategoryResponse[];
  onPress: (id: string) => void;
}

export function ItemCard({ item, categories, onPress }: ItemCardProps) {
  const category = categories.find((c) => c.id === item.category_id);
  const parsedPrice = item.estimated_price ? parseFloat(item.estimated_price) : NaN;
  const priceDisplay = !isNaN(parsedPrice) ? `${parsedPrice.toFixed(0)} \u20AC` : null;

  return (
    <Pressable onPress={() => onPress(item.id)} testID={`item-card-${item.id}`}>
      <Surface style={styles.surface} elevation={1}>
        <View style={styles.row}>
          <PriorityIndicator priority={item.priority} />
          <View style={styles.content}>
            <View style={styles.topRow}>
              <Text variant="titleSmall" style={styles.name} numberOfLines={1}>
                {item.name}
              </Text>
              {priceDisplay && (
                <Text variant="bodyMedium" style={styles.price}>
                  {priceDisplay}
                </Text>
              )}
            </View>
            <View style={styles.bottomRow}>
              {category && <CategoryChip category={category} />}
              <AgeBadge createdAt={item.created_at} size="small" />
            </View>
          </View>
        </View>
      </Surface>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  surface: {
    borderRadius: borderRadius.sm,
    padding: spacing.md,
    marginHorizontal: spacing.md,
    marginVertical: spacing.xs,
    backgroundColor: colors.surface,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
  content: {
    flex: 1,
    gap: spacing.xs,
  },
  topRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  name: {
    flex: 1,
    color: colors.text,
    fontWeight: '600',
  },
  price: {
    color: colors.textSecondary,
    marginLeft: spacing.sm,
  },
  bottomRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
});
