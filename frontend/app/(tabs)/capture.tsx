import { useState, useCallback, useEffect } from 'react';
import { View, StyleSheet, ScrollView, KeyboardAvoidingView, Platform } from 'react-native';
import { Text, TextInput, Button, Snackbar } from 'react-native-paper';
import { SafeAreaView } from 'react-native-safe-area-context';
import { MaterialCommunityIcons } from '@expo/vector-icons';
import { useTranslation } from 'react-i18next';

import { colors, spacing, borderRadius } from '@/src/theme';
import { useItemStore } from '@/src/stores/itemStore';
import { useCategoryStore } from '@/src/stores/categoryStore';
import { QuickCaptureInput, CategoryChip } from '@/src/components/items';
import type { ItemPriority } from '@/src/types/items';

export default function CaptureScreen() {
  const { t } = useTranslation();
  const createItem = useItemStore((s) => s.createItem);
  const isCreating = useItemStore((s) => s.isCreating);
  const categories = useCategoryStore((s) => s.categories);
  const fetchCategories = useCategoryStore((s) => s.fetchCategories);

  const [showDetails, setShowDetails] = useState(false);
  const [pendingName, setPendingName] = useState('');
  const [price, setPrice] = useState('');
  const [url, setUrl] = useState('');
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [priority, setPriority] = useState<ItemPriority>(2);
  const [notes, setNotes] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

  useEffect(() => {
    fetchCategories();
  }, [fetchCategories]);

  const resetDetails = useCallback(() => {
    setPendingName('');
    setPrice('');
    setUrl('');
    setSelectedCategoryId(null);
    setPriority(2);
    setNotes('');
    setShowDetails(false);
  }, []);

  const handleQuickSubmit = useCallback(
    async (name: string) => {
      try {
        await createItem({ name });
      } catch (e) {
        setErrorMessage(t('capture.errors.saveFailed'));
        throw e;
      }
    },
    [createItem, t],
  );

  const handleDetailSubmit = useCallback(async () => {
    const name = pendingName.trim();
    if (!name) {
      setErrorMessage(t('capture.errors.nameRequired'));
      return;
    }

    try {
      await createItem({
        name,
        estimated_price: price && !isNaN(parseFloat(price)) ? parseFloat(price) : undefined,
        url: url || undefined,
        category_id: selectedCategoryId ?? undefined,
        priority,
        description: notes || undefined,
      });
      resetDetails();
    } catch {
      setErrorMessage(t('capture.errors.saveFailed'));
    }
  }, [pendingName, price, url, selectedCategoryId, priority, notes, createItem, resetDetails, t]);

  const priorityOptions: { value: ItemPriority; label: string; color: string }[] = [
    { value: 1, label: t('capture.priority.low'), color: colors.textSecondary },
    { value: 2, label: t('capture.priority.medium'), color: colors.ageModerate },
    { value: 3, label: t('capture.priority.high'), color: colors.error },
  ];

  return (
    <SafeAreaView style={styles.safe} edges={['top']}>
      <KeyboardAvoidingView
        style={styles.flex}
        behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      >
        <ScrollView
          contentContainerStyle={styles.content}
          keyboardShouldPersistTaps="handled"
        >
          {!showDetails && (
            <>
              <MaterialCommunityIcons
                name="gift-outline"
                size={48}
                color={colors.primary}
                style={styles.heroIcon}
              />
              <Text variant="headlineSmall" style={styles.heroTitle}>
                {t('capture.placeholder')}
              </Text>
            </>
          )}

          {showDetails ? (
            <View style={styles.detailsContainer}>
              <TextInput
                testID="detail-name-input"
                mode="outlined"
                label={t('detail.nameLabel')}
                value={pendingName}
                onChangeText={setPendingName}
                autoFocus
                outlineColor={colors.inputBorder}
                activeOutlineColor={colors.primary}
                outlineStyle={styles.inputOutline}
                style={styles.input}
              />

              <Button
                onPress={() => setShowDetails(false)}
                mode="text"
                compact
                testID="hide-details-button"
              >
                {t('capture.hideDetails')}
              </Button>

              <TextInput
                testID="detail-price-input"
                mode="outlined"
                label={t('capture.priceLabel')}
                value={price}
                onChangeText={setPrice}
                keyboardType="decimal-pad"
                outlineColor={colors.inputBorder}
                activeOutlineColor={colors.primary}
                outlineStyle={styles.inputOutline}
                style={styles.input}
              />

              <TextInput
                testID="detail-url-input"
                mode="outlined"
                label={t('capture.urlLabel')}
                value={url}
                onChangeText={setUrl}
                keyboardType="url"
                autoCapitalize="none"
                outlineColor={colors.inputBorder}
                activeOutlineColor={colors.primary}
                outlineStyle={styles.inputOutline}
                style={styles.input}
              />

              {categories.length > 0 && (
                <View style={styles.section}>
                  <Text variant="bodyMedium" style={styles.sectionLabel}>
                    {t('capture.categoryLabel')}
                  </Text>
                  <ScrollView horizontal showsHorizontalScrollIndicator={false}>
                    <View style={styles.chipRow}>
                      {categories.map((cat) => (
                        <CategoryChip
                          key={cat.id}
                          category={cat}
                          selected={selectedCategoryId === cat.id}
                          onPress={() =>
                            setSelectedCategoryId(
                              selectedCategoryId === cat.id ? null : cat.id,
                            )
                          }
                        />
                      ))}
                    </View>
                  </ScrollView>
                </View>
              )}

              <View style={styles.section}>
                <Text variant="bodyMedium" style={styles.sectionLabel}>
                  {t('capture.priorityLabel')}
                </Text>
                <View style={styles.priorityRow}>
                  {priorityOptions.map((opt) => (
                    <Button
                      key={opt.value}
                      testID={`priority-${opt.value}`}
                      mode={priority === opt.value ? 'contained' : 'outlined'}
                      compact
                      onPress={() => setPriority(opt.value)}
                      buttonColor={priority === opt.value ? opt.color : undefined}
                      textColor={priority === opt.value ? '#FFFFFF' : opt.color}
                      style={styles.priorityButton}
                    >
                      {opt.label}
                    </Button>
                  ))}
                </View>
              </View>

              <TextInput
                testID="detail-notes-input"
                mode="outlined"
                label={t('capture.notesLabel')}
                value={notes}
                onChangeText={setNotes}
                multiline
                numberOfLines={3}
                outlineColor={colors.inputBorder}
                activeOutlineColor={colors.primary}
                outlineStyle={styles.inputOutline}
                style={styles.input}
              />

              <Button
                testID="detail-submit-button"
                mode="contained"
                onPress={handleDetailSubmit}
                loading={isCreating}
                disabled={isCreating || !pendingName.trim()}
                style={styles.submitButton}
              >
                {t('capture.submit')}
              </Button>
            </View>
          ) : (
            <>
              <View style={styles.inputWrapper}>
                <QuickCaptureInput onSubmit={handleQuickSubmit} isSubmitting={isCreating} />
              </View>

              <Button
                onPress={() => setShowDetails(true)}
                mode="text"
                compact
                testID="show-details-button"
              >
                {t('capture.addDetails')}
              </Button>
            </>
          )}
        </ScrollView>
      </KeyboardAvoidingView>

      <Snackbar
        visible={!!errorMessage}
        onDismiss={() => setErrorMessage('')}
        duration={3000}
        action={{ label: 'OK', onPress: () => setErrorMessage('') }}
      >
        {errorMessage}
      </Snackbar>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  flex: {
    flex: 1,
  },
  content: {
    flexGrow: 1,
    justifyContent: 'center',
    padding: spacing.lg,
  },
  heroIcon: {
    alignSelf: 'center',
    marginBottom: spacing.md,
  },
  heroTitle: {
    textAlign: 'center',
    color: colors.text,
    marginBottom: spacing.lg,
  },
  inputWrapper: {
    position: 'relative',
    marginBottom: spacing.md,
  },
  detailsContainer: {
    gap: spacing.md,
  },
  input: {
    backgroundColor: colors.inputBackground,
  },
  inputOutline: {
    borderRadius: borderRadius.sm,
  },
  section: {
    gap: spacing.sm,
  },
  sectionLabel: {
    color: colors.textSecondary,
  },
  chipRow: {
    flexDirection: 'row',
    gap: spacing.sm,
  },
  priorityRow: {
    flexDirection: 'row',
    gap: spacing.sm,
  },
  priorityButton: {
    flex: 1,
  },
  submitButton: {
    marginTop: spacing.sm,
    borderRadius: borderRadius.lg,
  },
});
