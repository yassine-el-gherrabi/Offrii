import { View, StyleSheet } from 'react-native';
import { Text } from 'react-native-paper';
import { useTranslation } from 'react-i18next';

import { colors, spacing } from '@/src/theme';

type Strength = 'weak' | 'good' | 'strong';

function getStrength(password: string): Strength {
  let score = 0;
  if (password.length >= 8) score++;
  if (password.length >= 12) score++;
  if (/[A-Z]/.test(password)) score++;
  if (/[0-9]/.test(password)) score++;
  if (/[^A-Za-z0-9]/.test(password)) score++;

  if (score <= 1) return 'weak';
  if (score <= 3) return 'good';
  return 'strong';
}

const STRENGTH_CONFIG: Record<Strength, { color: string; segments: number; labelKey: string }> = {
  weak: { color: colors.error, segments: 1, labelKey: 'auth.register.passwordStrength.weak' },
  good: { color: colors.ageModerate, segments: 2, labelKey: 'auth.register.passwordStrength.good' },
  strong: { color: colors.success, segments: 3, labelKey: 'auth.register.passwordStrength.strong' },
};

interface Props {
  password: string;
}

export default function PasswordStrengthIndicator({ password }: Props) {
  const { t } = useTranslation();

  if (password.length === 0) return null;

  const strength = getStrength(password);
  const config = STRENGTH_CONFIG[strength];

  return (
    <View style={styles.container}>
      <View style={styles.barRow} testID="strength-bar">
        {[0, 1, 2].map((i) => (
          <View
            key={i}
            style={[
              styles.segment,
              { backgroundColor: i < config.segments ? config.color : '#E5E7EB' },
              i < 2 && styles.segmentGap,
            ]}
          />
        ))}
      </View>
      <Text
        style={[styles.label, { color: config.color }]}
        testID="strength-label"
      >
        {t(config.labelKey)}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    paddingHorizontal: spacing.md,
    paddingTop: spacing.xs,
    paddingBottom: spacing.sm,
  },
  barRow: {
    flexDirection: 'row',
  },
  segment: {
    flex: 1,
    height: 4,
    borderRadius: 2,
  },
  segmentGap: {
    marginRight: spacing.xs,
  },
  label: {
    fontSize: 12,
    marginTop: 2,
  },
});
