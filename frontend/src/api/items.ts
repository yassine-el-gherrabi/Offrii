import { apiClient } from './client';
import type {
  ItemResponse,
  ItemsListResponse,
  CreateItemRequest,
  UpdateItemRequest,
  ListItemsParams,
} from '@/src/types/items';

export async function createItem(data: CreateItemRequest): Promise<ItemResponse> {
  const { data: item } = await apiClient.post<ItemResponse>('/items', data);
  return item;
}

export async function getItems(params?: ListItemsParams): Promise<ItemsListResponse> {
  const { data } = await apiClient.get<ItemsListResponse>('/items', { params });
  return data;
}

export async function getItem(id: string): Promise<ItemResponse> {
  const { data } = await apiClient.get<ItemResponse>(`/items/${id}`);
  return data;
}

export async function updateItem(id: string, data: UpdateItemRequest): Promise<ItemResponse> {
  const { data: item } = await apiClient.put<ItemResponse>(`/items/${id}`, data);
  return item;
}

export async function deleteItem(id: string): Promise<void> {
  await apiClient.delete(`/items/${id}`);
}
