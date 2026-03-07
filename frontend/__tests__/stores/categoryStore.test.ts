import { act } from '@testing-library/react-native';

jest.mock('@/src/api/categories', () => ({
  getCategories: jest.fn(),
}));

import { useCategoryStore } from '@/src/stores/categoryStore';
import * as categoriesApi from '@/src/api/categories';
import type { CategoryResponse } from '@/src/types/items';

const mockGetCategories = categoriesApi.getCategories as jest.MockedFunction<
  typeof categoriesApi.getCategories
>;

const MOCK_CATEGORIES: CategoryResponse[] = [
  {
    id: 'cat-1',
    name: 'Tech',
    icon: 'laptop',
    is_default: true,
    position: 0,
    created_at: '2025-01-01T00:00:00Z',
  },
  {
    id: 'cat-2',
    name: 'Mode',
    icon: null,
    is_default: false,
    position: 1,
    created_at: '2025-01-01T00:00:00Z',
  },
];

function resetStore() {
  useCategoryStore.setState({
    categories: [],
    isLoading: false,
    error: null,
  });
}

beforeEach(() => {
  jest.clearAllMocks();
  resetStore();
});

describe('useCategoryStore', () => {
  describe('fetchCategories', () => {
    it('fetches and stores categories', async () => {
      mockGetCategories.mockResolvedValueOnce(MOCK_CATEGORIES);

      await act(async () => {
        await useCategoryStore.getState().fetchCategories();
      });

      const state = useCategoryStore.getState();
      expect(state.categories).toEqual(MOCK_CATEGORIES);
      expect(state.isLoading).toBe(false);
    });

    it('sets error on failure', async () => {
      mockGetCategories.mockRejectedValueOnce(new Error('Network error'));

      await act(async () => {
        await useCategoryStore.getState().fetchCategories();
      });

      expect(useCategoryStore.getState().error).toBe('Network error');
      expect(useCategoryStore.getState().isLoading).toBe(false);
    });

    it('guards against duplicate fetches', async () => {
      useCategoryStore.setState({ isLoading: true });

      await act(async () => {
        await useCategoryStore.getState().fetchCategories();
      });

      expect(mockGetCategories).not.toHaveBeenCalled();
    });
  });
});
