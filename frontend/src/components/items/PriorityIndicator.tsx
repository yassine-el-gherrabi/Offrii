import { View, StyleSheet } from 'react-native';

import { colors } from '@/src/theme';

interface PriorityIndicatorProps {
  priority: number;
}

function priorityColor(priority: number): string {
  if (priority >= 3) return colors.error;
  if (priority === 2) return colors.ageModerate;
  return colors.textSecondary;
}

export function PriorityIndicator({ priority }: PriorityIndicatorProps) {
  return (
    <View
      testID="priority-indicator"
      accessibilityLabel={`Priority ${priority}`}
      accessibilityRole="text"
      style={[styles.dot, { backgroundColor: priorityColor(priority) }]}
    />
  );
}

const styles = StyleSheet.create({
  dot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
});
