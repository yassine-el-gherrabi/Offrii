import { useState, useRef, useCallback } from 'react';
import { StyleSheet, Animated } from 'react-native';
import { TextInput } from 'react-native-paper';
import { useTranslation } from 'react-i18next';
import * as Haptics from 'expo-haptics';

import { colors, spacing, borderRadius } from '@/src/theme';

interface QuickCaptureInputProps {
  onSubmit: (name: string) => Promise<void>;
  isSubmitting: boolean;
}

export function QuickCaptureInput({ onSubmit, isSubmitting }: QuickCaptureInputProps) {
  const { t } = useTranslation();
  const [text, setText] = useState('');
  const [showCheck, setShowCheck] = useState(false);
  const checkScale = useRef(new Animated.Value(0)).current;
  const checkOpacity = useRef(new Animated.Value(0)).current;

  const playSuccessAnimation = useCallback(() => {
    setShowCheck(true);
    checkScale.setValue(0);
    checkOpacity.setValue(0);

    Animated.sequence([
      // Scale in with spring
      Animated.parallel([
        Animated.spring(checkScale, {
          toValue: 1.2,
          useNativeDriver: true,
        }),
        Animated.timing(checkOpacity, {
          toValue: 1,
          duration: 100,
          useNativeDriver: true,
        }),
      ]),
      // Settle
      Animated.spring(checkScale, {
        toValue: 1,
        useNativeDriver: true,
      }),
      // Wait then fade out
      Animated.delay(200),
      Animated.timing(checkOpacity, {
        toValue: 0,
        duration: 200,
        useNativeDriver: true,
      }),
    ]).start(() => {
      setShowCheck(false);
    });
  }, [checkScale, checkOpacity]);

  const handleSubmit = useCallback(async () => {
    const trimmed = text.trim();
    if (!trimmed || isSubmitting) return;

    try {
      await onSubmit(trimmed);
      setText('');
      void Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success).catch(() => {});
      playSuccessAnimation();
    } catch {
      // Error handled by parent
    }
  }, [text, isSubmitting, onSubmit, playSuccessAnimation]);

  return (
    <>
      <TextInput
        testID="quick-capture-input"
        mode="outlined"
        value={text}
        onChangeText={setText}
        onSubmitEditing={handleSubmit}
        placeholder={t('capture.placeholder')}
        returnKeyType="done"
        autoFocus
        disabled={isSubmitting}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        outlineStyle={styles.inputOutline}
        right={<TextInput.Icon icon="arrow-right" onPress={handleSubmit} disabled={isSubmitting} />}
        style={styles.input}
      />
      {showCheck && (
        <Animated.Text
          testID="success-check"
          style={[
            styles.check,
            {
              transform: [{ scale: checkScale }],
              opacity: checkOpacity,
            },
          ]}
        >
          {'✓'}
        </Animated.Text>
      )}
    </>
  );
}

const styles = StyleSheet.create({
  input: {
    backgroundColor: colors.inputBackground,
  },
  inputOutline: {
    borderRadius: borderRadius.sm,
  },
  check: {
    position: 'absolute',
    alignSelf: 'center',
    fontSize: 48,
    color: colors.success,
    marginTop: spacing.md,
  },
});
