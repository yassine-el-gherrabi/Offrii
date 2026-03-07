import { Chip } from 'react-native-paper';

import type { CategoryResponse } from '@/src/types/items';

interface CategoryChipProps {
  category: CategoryResponse;
  selected?: boolean;
  onPress?: () => void;
}

export function CategoryChip({ category, selected = false, onPress }: CategoryChipProps) {
  return (
    <Chip
      testID={`category-chip-${category.id}`}
      selected={selected}
      onPress={onPress}
      icon={category.icon ?? undefined}
      mode={selected ? 'flat' : 'outlined'}
      compact
    >
      {category.name}
    </Chip>
  );
}
