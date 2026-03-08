import { ScrollView, StyleSheet, View } from 'react-native';
import { Text } from 'react-native-paper';
import { useTranslation } from 'react-i18next';
import { colors, spacing } from '@/src/theme';

export default function LegalNoticeScreen() {
  const { t } = useTranslation();

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text variant="headlineSmall" style={styles.heading}>
        {t('legal.legalNotice.title')}
      </Text>

      <Section title={t('legal.legalNotice.editor.title')}>
        <Text style={styles.body}>{t('legal.legalNotice.editor.body')}</Text>
      </Section>

      <Section title={t('legal.legalNotice.director.title')}>
        <Text style={styles.body}>{t('legal.legalNotice.director.body')}</Text>
      </Section>

      <Section title={t('legal.legalNotice.host.title')}>
        <Text style={styles.body}>{t('legal.legalNotice.host.body')}</Text>
      </Section>

      <Section title={t('legal.legalNotice.contact.title')}>
        <Text style={styles.body}>{t('legal.legalNotice.contact.body')}</Text>
      </Section>
    </ScrollView>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <View style={styles.section}>
      <Text variant="titleMedium" style={styles.sectionTitle}>
        {title}
      </Text>
      {children}
    </View>
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
