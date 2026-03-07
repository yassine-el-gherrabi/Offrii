import { apiClient } from './client';
import type { CategoryResponse } from '@/src/types/items';

export async function getCategories(): Promise<CategoryResponse[]> {
  const { data } = await apiClient.get<CategoryResponse[]>('/categories');
  return data;
}
