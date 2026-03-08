export const ROUTES = {
  HOME: '/(tabs)/capture',
  LOGIN: '/(auth)/login',
  REGISTER: '/(auth)/register',
  FORGOT_PASSWORD: '/(auth)/forgot-password',
  RESET_PASSWORD: '/(auth)/reset-password',
  ITEM_LIST: '/(tabs)/list',
  itemDetail: (id: string) => `/(tabs)/list/${id}` as const,
} as const;
