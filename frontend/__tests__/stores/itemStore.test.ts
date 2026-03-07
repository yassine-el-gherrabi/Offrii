import { act } from '@testing-library/react-native';

jest.mock('@/src/api/items', () => ({
  createItem: jest.fn(),
  getItems: jest.fn(),
  getItem: jest.fn(),
  updateItem: jest.fn(),
  deleteItem: jest.fn(),
}));

import { useItemStore } from '@/src/stores/itemStore';
import * as itemsApi from '@/src/api/items';
import type { ItemResponse, ItemsListResponse } from '@/src/types/items';

const mockGetItems = itemsApi.getItems as jest.MockedFunction<typeof itemsApi.getItems>;
const mockCreateItem = itemsApi.createItem as jest.MockedFunction<typeof itemsApi.createItem>;
const mockUpdateItem = itemsApi.updateItem as jest.MockedFunction<typeof itemsApi.updateItem>;
const mockDeleteItem = itemsApi.deleteItem as jest.MockedFunction<typeof itemsApi.deleteItem>;

const MOCK_ITEM: ItemResponse = {
  id: 'item-1',
  name: 'Test Item',
  description: null,
  url: null,
  estimated_price: null,
  priority: 2,
  category_id: null,
  status: 'active',
  purchased_at: null,
  created_at: '2025-01-01T00:00:00Z',
  updated_at: '2025-01-01T00:00:00Z',
};

const MOCK_LIST_RESPONSE: ItemsListResponse = {
  items: [MOCK_ITEM],
  total: 1,
  page: 1,
  per_page: 50,
};

function resetStore() {
  useItemStore.setState({
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
  });
}

beforeEach(() => {
  jest.clearAllMocks();
  resetStore();
});

describe('useItemStore', () => {
  describe('fetchItems', () => {
    it('fetches items and updates state', async () => {
      mockGetItems.mockResolvedValueOnce(MOCK_LIST_RESPONSE);

      await act(async () => {
        await useItemStore.getState().fetchItems();
      });

      const state = useItemStore.getState();
      expect(state.items).toEqual([MOCK_ITEM]);
      expect(state.total).toBe(1);
      expect(state.isLoading).toBe(false);
    });

    it('sets error on failure', async () => {
      mockGetItems.mockRejectedValueOnce(new Error('Network error'));

      await act(async () => {
        await useItemStore.getState().fetchItems();
      });

      expect(useItemStore.getState().error).toBe('Network error');
      expect(useItemStore.getState().isLoading).toBe(false);
    });
  });

  describe('createItem', () => {
    it('creates item and returns it', async () => {
      mockCreateItem.mockResolvedValueOnce(MOCK_ITEM);

      let result: ItemResponse | undefined;
      await act(async () => {
        result = await useItemStore.getState().createItem({ name: 'Test Item' });
      });

      expect(result).toEqual(MOCK_ITEM);
      expect(useItemStore.getState().isCreating).toBe(false);
    });

    it('sets error and throws on failure', async () => {
      mockCreateItem.mockRejectedValueOnce(new Error('Create failed'));

      await expect(
        useItemStore.getState().createItem({ name: 'Test' }),
      ).rejects.toThrow();

      expect(useItemStore.getState().error).toBe('Create failed');
      expect(useItemStore.getState().isCreating).toBe(false);
    });
  });

  describe('updateItem', () => {
    it('replaces the updated item in the list', async () => {
      const updated = { ...MOCK_ITEM, name: 'Updated' };
      useItemStore.setState({ items: [MOCK_ITEM], total: 1 });
      mockUpdateItem.mockResolvedValueOnce(updated);

      await act(async () => {
        await useItemStore.getState().updateItem('item-1', { name: 'Updated' });
      });

      expect(useItemStore.getState().items[0]!.name).toBe('Updated');
    });

    it('sets error and throws on failure', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 1 });
      mockUpdateItem.mockRejectedValueOnce(new Error('Update failed'));

      await expect(
        useItemStore.getState().updateItem('item-1', { name: 'X' }),
      ).rejects.toThrow();

      expect(useItemStore.getState().error).toBe('Update failed');
    });
  });

  describe('deleteItem (optimistic)', () => {
    it('removes item immediately and calls API', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 1 });
      mockDeleteItem.mockResolvedValueOnce(undefined);

      await act(async () => {
        await useItemStore.getState().deleteItem('item-1');
      });

      expect(useItemStore.getState().items).toEqual([]);
      expect(useItemStore.getState().total).toBe(0);
    });

    it('reverts on API failure', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 1 });
      mockDeleteItem.mockRejectedValueOnce(new Error('Delete failed'));

      await expect(
        useItemStore.getState().deleteItem('item-1'),
      ).rejects.toThrow();

      expect(useItemStore.getState().items).toEqual([MOCK_ITEM]);
      expect(useItemStore.getState().total).toBe(1);
      expect(useItemStore.getState().error).toBe('Delete failed');
    });

    it('does nothing for a non-existent item', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 1 });

      await act(async () => {
        await useItemStore.getState().deleteItem('non-existent');
      });

      expect(mockDeleteItem).not.toHaveBeenCalled();
      expect(useItemStore.getState().items).toEqual([MOCK_ITEM]);
    });
  });

  describe('markPurchased (optimistic)', () => {
    it('updates status immediately', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 1 });
      mockUpdateItem.mockResolvedValueOnce({ ...MOCK_ITEM, status: 'purchased' });

      await act(async () => {
        await useItemStore.getState().markPurchased('item-1');
      });

      expect(useItemStore.getState().items[0]!.status).toBe('purchased');
    });

    it('reverts on failure', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 1 });
      mockUpdateItem.mockRejectedValueOnce(new Error('Failed'));

      await expect(
        useItemStore.getState().markPurchased('item-1'),
      ).rejects.toThrow();

      expect(useItemStore.getState().items[0]!.status).toBe('active');
    });

    it('does nothing for already purchased items', async () => {
      const purchased = { ...MOCK_ITEM, status: 'purchased' as const };
      useItemStore.setState({ items: [purchased], total: 1 });

      await act(async () => {
        await useItemStore.getState().markPurchased('item-1');
      });

      expect(mockUpdateItem).not.toHaveBeenCalled();
    });
  });

  describe('loadMoreItems', () => {
    it('appends items on next page', async () => {
      const item2: ItemResponse = { ...MOCK_ITEM, id: 'item-2', name: 'Item 2' };
      useItemStore.setState({ items: [MOCK_ITEM], total: 2, page: 1 });
      mockGetItems.mockResolvedValueOnce({ items: [item2], total: 2, page: 2, per_page: 50 });

      await act(async () => {
        await useItemStore.getState().loadMoreItems();
      });

      expect(useItemStore.getState().items).toHaveLength(2);
      expect(useItemStore.getState().page).toBe(2);
    });

    it('does not load when already loading', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 2, page: 1, isLoading: true });

      await act(async () => {
        await useItemStore.getState().loadMoreItems();
      });

      expect(mockGetItems).not.toHaveBeenCalled();
    });

    it('does not load when all items loaded', async () => {
      useItemStore.setState({ items: [MOCK_ITEM], total: 1, page: 1 });

      await act(async () => {
        await useItemStore.getState().loadMoreItems();
      });

      expect(mockGetItems).not.toHaveBeenCalled();
    });
  });
});
