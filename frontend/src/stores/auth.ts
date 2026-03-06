import { create } from 'zustand';
import * as SecureStore from 'expo-secure-store';
import * as authApi from '@/src/api/auth';
import { setTokenGetter, setRefreshHandlers } from '@/src/api/client';
import type { UserResponse } from '@/src/types/auth';

const REFRESH_TOKEN_KEY = 'offrii_refresh_token';
const USER_DATA_KEY = 'offrii_user_data';

interface AuthState {
  accessToken: string | null;
  user: UserResponse | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, displayName?: string) => Promise<void>;
  logout: () => Promise<void>;
  restoreSession: () => Promise<void>;
}

export const useAuthStore = create<AuthState>((set, get) => {
  // Wire token getter to avoid circular imports
  setTokenGetter(() => get().accessToken);

  // Wire refresh handlers for 401 interceptor
  setRefreshHandlers({
    getRefreshToken: () => SecureStore.getItemAsync(REFRESH_TOKEN_KEY),
    onRefreshSuccess: async (accessToken, refreshToken) => {
      await SecureStore.setItemAsync(REFRESH_TOKEN_KEY, refreshToken);
      set({ accessToken, isAuthenticated: true });
    },
    onRefreshFailure: () => {
      // Clear state without calling API (we're already in a failed auth state)
      set({ accessToken: null, user: null, isAuthenticated: false });
      SecureStore.deleteItemAsync(REFRESH_TOKEN_KEY);
      SecureStore.deleteItemAsync(USER_DATA_KEY);
    },
  });

  return {
    accessToken: null,
    user: null,
    isAuthenticated: false,
    isLoading: true,

    login: async (email, password) => {
      const { tokens, user } = await authApi.login(email, password);
      await SecureStore.setItemAsync(REFRESH_TOKEN_KEY, tokens.refresh_token);
      await SecureStore.setItemAsync(USER_DATA_KEY, JSON.stringify(user));
      set({ accessToken: tokens.access_token, user, isAuthenticated: true });
    },

    register: async (email, password, displayName?) => {
      const { tokens, user } = await authApi.register(email, password, displayName);
      await SecureStore.setItemAsync(REFRESH_TOKEN_KEY, tokens.refresh_token);
      await SecureStore.setItemAsync(USER_DATA_KEY, JSON.stringify(user));
      set({ accessToken: tokens.access_token, user, isAuthenticated: true });
    },

    logout: async () => {
      // Call API BEFORE clearing state so the access token is still available
      try {
        await authApi.logout();
      } catch {
        // Best-effort — don't block logout on API failure
      }
      set({ accessToken: null, user: null, isAuthenticated: false });
      await SecureStore.deleteItemAsync(REFRESH_TOKEN_KEY);
      await SecureStore.deleteItemAsync(USER_DATA_KEY);
    },

    restoreSession: async () => {
      try {
        const refreshToken = await SecureStore.getItemAsync(REFRESH_TOKEN_KEY);
        if (!refreshToken) {
          set({ isLoading: false });
          return;
        }

        // Restore cached user data immediately
        const userJson = await SecureStore.getItemAsync(USER_DATA_KEY);
        const cachedUser: UserResponse | null = userJson ? JSON.parse(userJson) : null;

        const { tokens } = await authApi.refresh(refreshToken);
        await SecureStore.setItemAsync(REFRESH_TOKEN_KEY, tokens.refresh_token);
        set({
          accessToken: tokens.access_token,
          user: cachedUser,
          isAuthenticated: true,
          isLoading: false,
        });
      } catch {
        await SecureStore.deleteItemAsync(REFRESH_TOKEN_KEY);
        await SecureStore.deleteItemAsync(USER_DATA_KEY);
        set({ isLoading: false });
      }
    },
  };
});
