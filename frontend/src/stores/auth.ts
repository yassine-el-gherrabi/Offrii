import { Platform } from 'react-native';
import { create } from 'zustand';
import * as SecureStore from 'expo-secure-store';
import * as authApi from '@/src/api/auth';
import { setTokenGetter } from '@/src/api/client';
import type { UserResponse } from '@/src/types/auth';

const REFRESH_TOKEN_KEY = 'offrii_refresh_token';

// expo-secure-store is not available on web — fall back to localStorage
const tokenStorage = {
  async get(key: string): Promise<string | null> {
    if (Platform.OS === 'web') {
      return localStorage.getItem(key);
    }
    return SecureStore.getItemAsync(key);
  },
  async set(key: string, value: string): Promise<void> {
    if (Platform.OS === 'web') {
      localStorage.setItem(key, value);
      return;
    }
    await SecureStore.setItemAsync(key, value);
  },
  async remove(key: string): Promise<void> {
    if (Platform.OS === 'web') {
      localStorage.removeItem(key);
      return;
    }
    await SecureStore.deleteItemAsync(key);
  },
};

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

  return {
    accessToken: null,
    user: null,
    isAuthenticated: false,
    isLoading: true,

    login: async (email, password) => {
      const { tokens, user } = await authApi.login(email, password);
      await tokenStorage.set(REFRESH_TOKEN_KEY, tokens.refresh_token);
      set({ accessToken: tokens.access_token, user, isAuthenticated: true });
    },

    register: async (email, password, displayName?) => {
      const { tokens, user } = await authApi.register(email, password, displayName);
      await tokenStorage.set(REFRESH_TOKEN_KEY, tokens.refresh_token);
      set({ accessToken: tokens.access_token, user, isAuthenticated: true });
    },

    logout: async () => {
      const { accessToken } = get();
      set({ accessToken: null, user: null, isAuthenticated: false });
      await tokenStorage.remove(REFRESH_TOKEN_KEY);
      if (accessToken) {
        // Fire-and-forget — don't block logout on API
        authApi.logout(accessToken).catch(() => {});
      }
    },

    restoreSession: async () => {
      try {
        const refreshToken = await tokenStorage.get(REFRESH_TOKEN_KEY);
        if (!refreshToken) {
          set({ isLoading: false });
          return;
        }
        const { tokens } = await authApi.refresh(refreshToken);
        await tokenStorage.set(REFRESH_TOKEN_KEY, tokens.refresh_token);
        set({ accessToken: tokens.access_token, isAuthenticated: true, isLoading: false });
      } catch {
        await tokenStorage.remove(REFRESH_TOKEN_KEY);
        set({ isLoading: false });
      }
    },
  };
});
