import { create } from 'zustand';
import * as categoriesApi from '@/src/api/categories';
import type { CategoryResponse } from '@/src/types/items';

interface CategoryState {
  categories: CategoryResponse[];
  isLoading: boolean;
  error: string | null;

  fetchCategories: () => Promise<void>;
}

export const useCategoryStore = create<CategoryState>((set, get) => ({
  categories: [],
  isLoading: false,
  error: null,

  fetchCategories: async () => {
    if (get().isLoading) return;
    set({ isLoading: true, error: null });
    try {
      const categories = await categoriesApi.getCategories();
      set({ categories, isLoading: false });
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Failed to load categories';
      set({ error: message, isLoading: false });
    }
  },
}));
