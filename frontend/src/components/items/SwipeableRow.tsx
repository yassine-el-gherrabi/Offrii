import { useRef } from 'react';
import { StyleSheet, View, Animated } from 'react-native';
import { Swipeable } from 'react-native-gesture-handler';
import { MaterialCommunityIcons } from '@expo/vector-icons';
import { Text } from 'react-native-paper';
import * as Haptics from 'expo-haptics';
import { useTranslation } from 'react-i18next';

import { colors, spacing } from '@/src/theme';

interface SwipeableRowProps {
  onSwipeLeft: () => void;
  onSwipeRight: () => void;
  children: React.ReactNode;
}

export function SwipeableRow({ onSwipeLeft, onSwipeRight, children }: SwipeableRowProps) {
  const { t } = useTranslation();
  const swipeableRef = useRef<Swipeable>(null);

  const renderRightActions = (_progress: Animated.AnimatedInterpolation<number>) => (
    <View style={[styles.action, styles.rightAction]}>
      <MaterialCommunityIcons name="check" size={24} color="#FFFFFF" />
      <Text style={styles.actionText}>{t('list.swipe.purchased')}</Text>
    </View>
  );

  const renderLeftActions = (_progress: Animated.AnimatedInterpolation<number>) => (
    <View style={[styles.action, styles.leftAction]}>
      <MaterialCommunityIcons name="delete" size={24} color="#FFFFFF" />
      <Text style={styles.actionText}>{t('list.swipe.delete')}</Text>
    </View>
  );

  const handleSwipeOpen = (direction: 'left' | 'right') => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium).catch(() => {});
    swipeableRef.current?.close();
    if (direction === 'right') onSwipeRight();
    else onSwipeLeft();
  };

  return (
    <Swipeable
      ref={swipeableRef}
      renderRightActions={renderRightActions}
      renderLeftActions={renderLeftActions}
      onSwipeableOpen={(direction) => handleSwipeOpen(direction)}
      testID="swipeable-row"
    >
      {children}
    </Swipeable>
  );
}

const styles = StyleSheet.create({
  action: {
    justifyContent: 'center',
    alignItems: 'center',
    width: 100,
    paddingHorizontal: spacing.md,
  },
  rightAction: {
    backgroundColor: colors.success,
  },
  leftAction: {
    backgroundColor: colors.error,
  },
  actionText: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '600',
    marginTop: spacing.xs,
  },
});
