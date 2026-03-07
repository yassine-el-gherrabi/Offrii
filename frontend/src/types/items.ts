export type ItemStatus = 'active' | 'purchased';
export type ItemPriority = 1 | 2 | 3;
export type SortField = 'name' | 'created_at' | 'updated_at' | 'priority';
export type SortOrder = 'asc' | 'desc';

export interface ItemResponse {
  id: string;
  name: string;
  description: string | null;
  url: string | null;
  estimated_price: string | null;
  priority: number;
  category_id: string | null;
  status: ItemStatus;
  purchased_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface ItemsListResponse {
  items: ItemResponse[];
  total: number;
  page: number;
  per_page: number;
}

export interface CreateItemRequest {
  name: string;
  description?: string;
  url?: string;
  estimated_price?: number;
  priority?: ItemPriority;
  category_id?: string;
}

export interface UpdateItemRequest {
  name?: string;
  description?: string;
  url?: string;
  estimated_price?: number;
  priority?: ItemPriority;
  category_id?: string | null;
  status?: ItemStatus;
}

export interface ListItemsParams {
  status?: ItemStatus;
  category_id?: string;
  sort?: SortField;
  order?: SortOrder;
  page?: number;
  per_page?: number;
}

export interface CategoryResponse {
  id: string;
  name: string;
  icon: string | null;
  is_default: boolean;
  position: number;
  created_at: string;
}
