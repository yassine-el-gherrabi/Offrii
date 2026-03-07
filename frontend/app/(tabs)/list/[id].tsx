import { useState, useEffect, useCallback } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
  Linking,
} from 'react-native';
import {
  Text,
  TextInput,
  Button,
  Dialog,
  Portal,
  Snackbar,
  ActivityIndicator,
} from 'react-native-paper';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useLocalSearchParams, router } from 'expo-router';
import { useTranslation } from 'react-i18next';

import { colors, spacing, borderRadius } from '@/src/theme';
import { AgeBadge, CategoryChip } from '@/src/components/items';
import { useCategoryStore } from '@/src/stores/categoryStore';
import { useItemStore } from '@/src/stores/itemStore';
import * as itemsApi from '@/src/api/items';
import type { ItemResponse, ItemPriority } from '@/src/types/items';

export default function ItemDetailScreen() {
  const params = useLocalSearchParams<{ id: string }>();
  const id = typeof params.id === 'string' ? params.id : params.id?.[0];
  const { t } = useTranslation();

  const categories = useCategoryStore((s) => s.categories);
  const fetchCategories = useCategoryStore((s) => s.fetchCategories);
  const storeUpdateItem = useItemStore((s) => s.updateItem);
  const storeDeleteItem = useItemStore((s) => s.deleteItem);

  const [item, setItem] = useState<ItemResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [notFound, setNotFound] = useState(false);

  // Form state
  const [name, setName] = useState('');
  const [price, setPrice] = useState('');
  const [url, setUrl] = useState('');
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [priority, setPriority] = useState<ItemPriority>(2);
  const [notes, setNotes] = useState('');

  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [snackMessage, setSnackMessage] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    fetchCategories();
  }, [fetchCategories]);

  useEffect(() => {
    if (!id) {
      setNotFound(true);
      setLoading(false);
      return;
    }
    let cancelled = false;
    (async () => {
      try {
        const data = await itemsApi.getItem(id);
        if (cancelled) return;
        setItem(data);
        setName(data.name);
        setPrice(data.estimated_price ? parseFloat(data.estimated_price).toString() : '');
        setUrl(data.url ?? '');
        setSelectedCategoryId(data.category_id);
        setPriority(data.priority as ItemPriority);
        setNotes(data.description ?? '');
      } catch {
        if (!cancelled) setNotFound(true);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id]);

  const hasChanges = useCallback(() => {
    if (!item) return false;
    return (
      name !== item.name ||
      price !== (item.estimated_price ? parseFloat(item.estimated_price).toString() : '') ||
      url !== (item.url ?? '') ||
      selectedCategoryId !== item.category_id ||
      priority !== item.priority ||
      notes !== (item.description ?? '')
    );
  }, [item, name, price, url, selectedCategoryId, priority, notes]);

  const handleSave = useCallback(async () => {
    if (!item || !name.trim()) return;
    setIsSaving(true);
    try {
      const updated = await storeUpdateItem(item.id, {
        name: name.trim(),
        estimated_price: price && !isNaN(parseFloat(price)) ? parseFloat(price) : undefined,
        url: url || undefined,
        category_id: selectedCategoryId,
        priority,
        description: notes || undefined,
      });
      setItem(updated);
      setSnackMessage(t('detail.saved'));
    } catch {
      setSnackMessage(t('detail.errors.saveFailed'));
    } finally {
      setIsSaving(false);
    }
  }, [item, name, price, url, selectedCategoryId, priority, notes, storeUpdateItem, t]);

  const handleToggleStatus = useCallback(async () => {
    if (!item) return;
    const newStatus = item.status === 'active' ? 'purchased' : 'active';
    try {
      await storeUpdateItem(item.id, { status: newStatus });
      router.back();
    } catch {
      setSnackMessage(t('detail.errors.statusFailed'));
    }
  }, [item, storeUpdateItem, t]);

  const handleDelete = useCallback(async () => {
    if (!item) return;
    setShowDeleteDialog(false);
    try {
      await storeDeleteItem(item.id);
      router.back();
    } catch {
      setSnackMessage(t('detail.errors.deleteFailed'));
    }
  }, [item, storeDeleteItem, t]);

  const handleOpenUrl = useCallback(async () => {
    if (!url) return;
    const fullUrl = url.startsWith('http') ? url : `https://${url}`;
    try {
      await Linking.openURL(fullUrl);
    } catch {
      setSnackMessage(t('detail.errors.invalidUrl'));
    }
  }, [url, t]);

  const priorityOptions: { value: ItemPriority; label: string; color: string }[] = [
    { value: 1, label: t('capture.priority.low'), color: colors.textSecondary },
    { value: 2, label: t('capture.priority.medium'), color: colors.ageModerate },
    { value: 3, label: t('capture.priority.high'), color: colors.error },
  ];

  if (loading) {
    return (
      <View style={styles.centered}>
        <ActivityIndicator size="large" />
      </View>
    );
  }

  if (notFound || !item) {
    return (
      <View style={styles.centered}>
        <Text variant="titleMedium">{t('detail.errors.notFound')}</Text>
        <Button mode="text" onPress={() => router.back()} style={styles.backButton}>
          {t('common.back')}
        </Button>
      </View>
    );
  }

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
          {/* Age badge */}
          <View style={styles.ageBadgeContainer}>
            <AgeBadge createdAt={item.created_at} size="medium" />
          </View>

          {/* Name */}
          <TextInput
            testID="detail-name"
            mode="outlined"
            label={t('detail.nameLabel')}
            value={name}
            onChangeText={setName}
            outlineColor={colors.inputBorder}
            activeOutlineColor={colors.primary}
            outlineStyle={styles.inputOutline}
            style={styles.input}
          />

          {/* Price */}
          <TextInput
            testID="detail-price"
            mode="outlined"
            label={t('detail.priceLabel')}
            value={price}
            onChangeText={setPrice}
            keyboardType="decimal-pad"
            outlineColor={colors.inputBorder}
            activeOutlineColor={colors.primary}
            outlineStyle={styles.inputOutline}
            style={styles.input}
          />

          {/* URL */}
          <View style={styles.urlRow}>
            <TextInput
              testID="detail-url"
              mode="outlined"
              label={t('detail.urlLabel')}
              value={url}
              onChangeText={setUrl}
              keyboardType="url"
              autoCapitalize="none"
              outlineColor={colors.inputBorder}
              activeOutlineColor={colors.primary}
              outlineStyle={styles.inputOutline}
              style={[styles.input, styles.urlInput]}
            />
            {url ? (
              <Button
                testID="open-url-button"
                mode="outlined"
                onPress={handleOpenUrl}
                compact
                style={styles.urlButton}
              >
                {t('detail.openUrl')}
              </Button>
            ) : null}
          </View>

          {/* Categories */}
          {categories.length > 0 && (
            <View style={styles.section}>
              <Text variant="bodyMedium" style={styles.sectionLabel}>
                {t('detail.categoryLabel')}
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

          {/* Priority */}
          <View style={styles.section}>
            <Text variant="bodyMedium" style={styles.sectionLabel}>
              {t('detail.priorityLabel')}
            </Text>
            <View style={styles.priorityRow}>
              {priorityOptions.map((opt) => (
                <Button
                  key={opt.value}
                  testID={`detail-priority-${opt.value}`}
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

          {/* Notes */}
          <TextInput
            testID="detail-notes"
            mode="outlined"
            label={t('detail.notesLabel')}
            value={notes}
            onChangeText={setNotes}
            multiline
            numberOfLines={4}
            outlineColor={colors.inputBorder}
            activeOutlineColor={colors.primary}
            outlineStyle={styles.inputOutline}
            style={styles.input}
          />

          {/* Actions */}
          <Button
            testID="save-button"
            mode="contained"
            onPress={handleSave}
            loading={isSaving}
            disabled={isSaving || !hasChanges() || !name.trim()}
            style={styles.actionButton}
          >
            {t('detail.save')}
          </Button>

          <Button
            testID="toggle-status-button"
            mode="outlined"
            onPress={handleToggleStatus}
            style={styles.actionButton}
          >
            {item.status === 'active'
              ? t('detail.markPurchased')
              : t('detail.markActive')}
          </Button>

          <Button
            testID="delete-button"
            mode="text"
            textColor={colors.error}
            onPress={() => setShowDeleteDialog(true)}
          >
            {t('detail.delete')}
          </Button>
        </ScrollView>
      </KeyboardAvoidingView>

      <Portal>
        <Dialog visible={showDeleteDialog} onDismiss={() => setShowDeleteDialog(false)}>
          <Dialog.Title>{t('list.deleteConfirm.title')}</Dialog.Title>
          <Dialog.Content>
            <Text variant="bodyMedium">{t('list.deleteConfirm.message')}</Text>
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setShowDeleteDialog(false)}>
              {t('list.deleteConfirm.cancel')}
            </Button>
            <Button onPress={handleDelete} textColor={colors.error}>
              {t('list.deleteConfirm.confirm')}
            </Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>

      <Snackbar
        visible={!!snackMessage}
        onDismiss={() => setSnackMessage('')}
        duration={3000}
      >
        {snackMessage}
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
  centered: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: colors.background,
  },
  content: {
    padding: spacing.lg,
    gap: spacing.md,
  },
  ageBadgeContainer: {
    alignItems: 'center',
    marginBottom: spacing.sm,
  },
  input: {
    backgroundColor: colors.inputBackground,
  },
  inputOutline: {
    borderRadius: borderRadius.sm,
  },
  urlRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
  },
  urlInput: {
    flex: 1,
  },
  urlButton: {
    marginTop: spacing.xs,
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
  actionButton: {
    borderRadius: borderRadius.lg,
  },
  backButton: {
    marginTop: spacing.md,
  },
});
