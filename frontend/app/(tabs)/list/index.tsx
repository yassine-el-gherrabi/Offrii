import { useCallback, useEffect, useState } from 'react';
import { View, StyleSheet, FlatList, ScrollView } from 'react-native';
import {
  SegmentedButtons,
  Chip,
  Text,
  ActivityIndicator,
  Dialog,
  Portal,
  Button,
  Snackbar,
  Menu,
  IconButton,
} from 'react-native-paper';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useTranslation } from 'react-i18next';
import { router, useFocusEffect } from 'expo-router';

import { colors, spacing } from '@/src/theme';
import { useItemStore } from '@/src/stores/itemStore';
import { useCategoryStore } from '@/src/stores/categoryStore';
import { ItemCard, SwipeableRow, EmptyState } from '@/src/components/items';
import type { ItemResponse, ItemStatus, SortField } from '@/src/types/items';

const SORT_OPTIONS: { value: SortField; labelKey: string }[] = [
  { value: 'created_at', labelKey: 'list.sort.created_at' },
  { value: 'name', labelKey: 'list.sort.name' },
  { value: 'priority', labelKey: 'list.sort.priority' },
  { value: 'updated_at', labelKey: 'list.sort.updated_at' },
];

export default function ListScreen() {
  const { t } = useTranslation();

  const items = useItemStore((s) => s.items);
  const total = useItemStore((s) => s.total);
  const isLoading = useItemStore((s) => s.isLoading);
  const isRefreshing = useItemStore((s) => s.isRefreshing);
  const error = useItemStore((s) => s.error);
  const statusFilter = useItemStore((s) => s.statusFilter);
  const categoryFilter = useItemStore((s) => s.categoryFilter);
  const sortField = useItemStore((s) => s.sortField);
  const sortOrder = useItemStore((s) => s.sortOrder);
  const fetchItems = useItemStore((s) => s.fetchItems);
  const refreshItems = useItemStore((s) => s.refreshItems);
  const loadMoreItems = useItemStore((s) => s.loadMoreItems);
  const setStatusFilter = useItemStore((s) => s.setStatusFilter);
  const setCategoryFilter = useItemStore((s) => s.setCategoryFilter);
  const setSortField = useItemStore((s) => s.setSortField);
  const setSortOrder = useItemStore((s) => s.setSortOrder);
  const deleteItem = useItemStore((s) => s.deleteItem);
  const markPurchased = useItemStore((s) => s.markPurchased);
  const clearError = useItemStore((s) => s.clearError);

  const categories = useCategoryStore((s) => s.categories);
  const fetchCategories = useCategoryStore((s) => s.fetchCategories);

  const [deleteTarget, setDeleteTarget] = useState<ItemResponse | null>(null);
  const [snackMessage, setSnackMessage] = useState('');
  const [sortMenuVisible, setSortMenuVisible] = useState(false);

  useFocusEffect(
    useCallback(() => {
      fetchItems();
      fetchCategories();
    }, [fetchItems, fetchCategories]),
  );

  useEffect(() => {
    if (error) setSnackMessage(error);
  }, [error]);

  const handleItemPress = useCallback((id: string) => {
    router.push(`/(tabs)/list/${id}`);
  }, []);

  const handleMarkPurchased = useCallback(
    async (item: ItemResponse) => {
      try {
        await markPurchased(item.id);
      } catch {
        // Error set in store
      }
    },
    [markPurchased],
  );

  const handleDeleteConfirm = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteItem(deleteTarget.id);
    } catch {
      // Error set in store
    }
    setDeleteTarget(null);
  }, [deleteTarget, deleteItem]);

  const renderItem = useCallback(
    ({ item }: { item: ItemResponse }) => (
      <SwipeableRow
        onSwipeLeft={() => handleMarkPurchased(item)}
        onSwipeRight={() => setDeleteTarget(item)}
      >
        <ItemCard item={item} categories={categories} onPress={handleItemPress} />
      </SwipeableRow>
    ),
    [categories, handleItemPress, handleMarkPurchased],
  );

  const renderFooter = useCallback(() => {
    if (!isLoading || items.length === 0) return null;
    return (
      <View style={styles.footer}>
        <ActivityIndicator size="small" />
      </View>
    );
  }, [isLoading, items.length]);

  return (
    <SafeAreaView style={styles.safe} edges={['top']}>
      {/* Status toggle */}
      <View style={styles.segmentContainer}>
        <SegmentedButtons
          value={statusFilter}
          onValueChange={(v) => setStatusFilter(v as ItemStatus)}
          buttons={[
            { value: 'active', label: t('list.active'), testID: 'filter-active' },
            { value: 'purchased', label: t('list.purchased'), testID: 'filter-purchased' },
          ]}
        />
      </View>

      {/* Category chips */}
      {categories.length > 0 && (
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          contentContainerStyle={styles.chipContainer}
          style={styles.chipScroll}
        >
          <Chip
            testID="category-all"
            selected={categoryFilter === null}
            onPress={() => setCategoryFilter(null)}
            mode={categoryFilter === null ? 'flat' : 'outlined'}
            compact
          >
            {t('list.allCategories')}
          </Chip>
          {categories.map((cat) => (
            <Chip
              key={cat.id}
              testID={`category-filter-${cat.id}`}
              selected={categoryFilter === cat.id}
              onPress={() => setCategoryFilter(categoryFilter === cat.id ? null : cat.id)}
              mode={categoryFilter === cat.id ? 'flat' : 'outlined'}
              icon={cat.icon ?? undefined}
              compact
            >
              {cat.name}
            </Chip>
          ))}
        </ScrollView>
      )}

      {/* Sort controls */}
      <View style={styles.sortRow}>
        <Menu
          visible={sortMenuVisible}
          onDismiss={() => setSortMenuVisible(false)}
          anchor={
            <Button
              mode="text"
              compact
              onPress={() => setSortMenuVisible(true)}
              testID="sort-menu-button"
              icon="sort"
            >
              {t('list.sortBy')}
            </Button>
          }
        >
          {SORT_OPTIONS.map((opt) => (
            <Menu.Item
              key={opt.value}
              title={t(opt.labelKey)}
              onPress={() => {
                setSortField(opt.value);
                setSortMenuVisible(false);
              }}
              leadingIcon={sortField === opt.value ? 'check' : undefined}
            />
          ))}
        </Menu>
        <IconButton
          testID="sort-order-toggle"
          icon={sortOrder === 'desc' ? 'sort-descending' : 'sort-ascending'}
          size={20}
          onPress={() => setSortOrder(sortOrder === 'desc' ? 'asc' : 'desc')}
        />
      </View>

      {/* Item list */}
      <FlatList
        testID="items-list"
        data={items}
        keyExtractor={(item) => item.id}
        renderItem={renderItem}
        ListEmptyComponent={
          !isLoading ? <EmptyState status={statusFilter} /> : null
        }
        ListFooterComponent={renderFooter}
        refreshing={isRefreshing}
        onRefresh={refreshItems}
        onEndReached={() => {
          if (items.length < total) loadMoreItems();
        }}
        onEndReachedThreshold={0.5}
        contentContainerStyle={items.length === 0 ? styles.emptyList : undefined}
      />

      {/* Delete confirmation dialog */}
      <Portal>
        <Dialog visible={!!deleteTarget} onDismiss={() => setDeleteTarget(null)}>
          <Dialog.Title>{t('list.deleteConfirm.title')}</Dialog.Title>
          <Dialog.Content>
            <Text variant="bodyMedium">{t('list.deleteConfirm.message')}</Text>
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setDeleteTarget(null)}>
              {t('list.deleteConfirm.cancel')}
            </Button>
            <Button onPress={handleDeleteConfirm} textColor={colors.error}>
              {t('list.deleteConfirm.confirm')}
            </Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>

      <Snackbar
        visible={!!snackMessage}
        onDismiss={() => { setSnackMessage(''); clearError(); }}
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
  segmentContainer: {
    paddingHorizontal: spacing.md,
    paddingTop: spacing.md,
    paddingBottom: spacing.sm,
  },
  chipScroll: {
    flexGrow: 0,
  },
  chipContainer: {
    paddingHorizontal: spacing.md,
    gap: spacing.sm,
    paddingBottom: spacing.sm,
    alignItems: 'center',
  },
  sortRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: spacing.md,
    paddingBottom: spacing.sm,
  },
  footer: {
    paddingVertical: spacing.md,
    alignItems: 'center',
  },
  emptyList: {
    flexGrow: 1,
  },
});
