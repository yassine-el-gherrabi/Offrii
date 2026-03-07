import { create } from 'zustand';
import * as itemsApi from '@/src/api/items';
import type {
  ItemResponse,
  CreateItemRequest,
  UpdateItemRequest,
  ItemStatus,
  SortField,
  SortOrder,
} from '@/src/types/items';

interface ItemState {
  items: ItemResponse[];
  total: number;
  page: number;
  perPage: number;

  statusFilter: ItemStatus;
  categoryFilter: string | null;
  sortField: SortField;
  sortOrder: SortOrder;

  isLoading: boolean;
  isRefreshing: boolean;
  isCreating: boolean;
  error: string | null;

  fetchItems: () => Promise<void>;
  refreshItems: () => Promise<void>;
  loadMoreItems: () => Promise<void>;
  createItem: (data: CreateItemRequest) => Promise<ItemResponse>;
  updateItem: (id: string, data: UpdateItemRequest) => Promise<ItemResponse>;
  deleteItem: (id: string) => Promise<void>;
  markPurchased: (id: string) => Promise<void>;

  clearError: () => void;
  setStatusFilter: (s: ItemStatus) => void;
  setCategoryFilter: (id: string | null) => void;
  setSortField: (f: SortField) => void;
  setSortOrder: (o: SortOrder) => void;
}

export const useItemStore = create<ItemState>((set, get) => ({
  items: [],
  total: 0,
  page: 1,
  perPage: 50,

  statusFilter: 'active',
  categoryFilter: null,
  sortField: 'created_at',
  sortOrder: 'desc',

  isLoading: false,
  isRefreshing: false,
  isCreating: false,
  error: null,

  fetchItems: async () => {
    const { statusFilter, categoryFilter, sortField, sortOrder, perPage } = get();
    set({ isLoading: true, error: null });
    try {
      const result = await itemsApi.getItems({
        status: statusFilter,
        category_id: categoryFilter ?? undefined,
        sort: sortField,
        order: sortOrder,
        page: 1,
        per_page: perPage,
      });
      set({ items: result.items, total: result.total, page: 1, isLoading: false });
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Failed to load items';
      set({ error: message, isLoading: false });
    }
  },

  refreshItems: async () => {
    const { statusFilter, categoryFilter, sortField, sortOrder, perPage } = get();
    set({ isRefreshing: true, error: null });
    try {
      const result = await itemsApi.getItems({
        status: statusFilter,
        category_id: categoryFilter ?? undefined,
        sort: sortField,
        order: sortOrder,
        page: 1,
        per_page: perPage,
      });
      set({ items: result.items, total: result.total, page: 1, isRefreshing: false });
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Failed to refresh items';
      set({ error: message, isRefreshing: false });
    }
  },

  loadMoreItems: async () => {
    const { items, total, page, perPage, statusFilter, categoryFilter, sortField, sortOrder, isLoading } = get();
    if (isLoading || items.length >= total) return;
    set({ isLoading: true });
    try {
      const nextPage = page + 1;
      const result = await itemsApi.getItems({
        status: statusFilter,
        category_id: categoryFilter ?? undefined,
        sort: sortField,
        order: sortOrder,
        page: nextPage,
        per_page: perPage,
      });
      set((state) => ({
        items: [...state.items, ...result.items],
        total: result.total,
        page: nextPage,
        isLoading: false,
      }));
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Failed to load more items';
      set({ error: message, isLoading: false });
    }
  },

  createItem: async (data) => {
    set({ isCreating: true, error: null });
    try {
      const item = await itemsApi.createItem(data);
      set({ isCreating: false });
      return item;
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Failed to create item';
      set({ error: message, isCreating: false });
      throw e;
    }
  },

  updateItem: async (id, data) => {
    set({ error: null });
    try {
      const updated = await itemsApi.updateItem(id, data);
      set((state) => ({
        items: state.items.map((item) => (item.id === id ? updated : item)),
      }));
      return updated;
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Failed to update item';
      set({ error: message });
      throw e;
    }
  },

  deleteItem: async (id) => {
    const { items } = get();
    const removed = items.find((item) => item.id === id);
    if (!removed) return;

    // Optimistic: remove immediately
    set({
      items: items.filter((item) => item.id !== id),
      total: get().total - 1,
      error: null,
    });

    try {
      await itemsApi.deleteItem(id);
    } catch (e) {
      // Revert on failure — append only if not already back (e.g. via concurrent refresh)
      const message = e instanceof Error ? e.message : 'Failed to delete item';
      set((state) => ({
        items: state.items.some((item) => item.id === removed.id)
          ? state.items
          : [...state.items, removed],
        total: state.items.some((item) => item.id === removed.id)
          ? state.total
          : state.total + 1,
        error: message,
      }));
      throw e;
    }
  },

  markPurchased: async (id) => {
    const { items } = get();
    const original = items.find((item) => item.id === id);
    if (!original || original.status === 'purchased') return;

    // Optimistic: update status immediately
    set({
      items: items.map((item) =>
        item.id === id ? { ...item, status: 'purchased' as ItemStatus, purchased_at: new Date().toISOString() } : item,
      ),
      error: null,
    });

    try {
      await itemsApi.updateItem(id, { status: 'purchased' });
    } catch (e) {
      // Revert on failure
      const message = e instanceof Error ? e.message : 'Failed to mark as purchased';
      set((state) => ({
        items: state.items.map((item) => (item.id === id ? original : item)),
        error: message,
      }));
      throw e;
    }
  },

  clearError: () => set({ error: null }),
  setStatusFilter: (s) => {
    set({ statusFilter: s });
    void get().fetchItems();
  },
  setCategoryFilter: (id) => {
    set({ categoryFilter: id });
    void get().fetchItems();
  },
  setSortField: (f) => {
    set({ sortField: f });
    void get().fetchItems();
  },
  setSortOrder: (o) => {
    set({ sortOrder: o });
    void get().fetchItems();
  },
}));
