export const ROUTES = {
  HOME: '/(tabs)/capture',
  LOGIN: '/(auth)/login',
  REGISTER: '/(auth)/register',
  ITEM_LIST: '/(tabs)/list',
  itemDetail: (id: string) => `/(tabs)/list/${id}` as const,
} as const;
