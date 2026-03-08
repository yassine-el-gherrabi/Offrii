import { ScrollView, StyleSheet, View } from 'react-native';
import { Text } from 'react-native-paper';
import { useTranslation } from 'react-i18next';
import { colors, spacing } from '@/src/theme';

const SECTIONS = [
  'controller',
  'dataCollected',
  'purposes',
  'legalBasis',
  'retention',
  'hosting',
  'rights',
  'cookies',
  'changes',
  'dpo',
] as const;

export default function PrivacyPolicyScreen() {
  const { t } = useTranslation();

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text variant="headlineSmall" style={styles.heading}>
        {t('legal.privacyPolicy.title')}
      </Text>

      {SECTIONS.map((key) => (
        <View key={key} style={styles.section}>
          <Text variant="titleMedium" style={styles.sectionTitle}>
            {t(`legal.privacyPolicy.${key}.title`)}
          </Text>
          <Text style={styles.body}>
            {t(`legal.privacyPolicy.${key}.body`)}
          </Text>
        </View>
      ))}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    padding: spacing.lg,
    paddingBottom: spacing.xl * 2,
  },
  heading: {
    color: colors.text,
    marginBottom: spacing.lg,
  },
  section: {
    marginBottom: spacing.lg,
  },
  sectionTitle: {
    color: colors.text,
    marginBottom: spacing.sm,
    fontWeight: '600',
  },
  body: {
    color: colors.textSecondary,
    lineHeight: 22,
  },
});
