import { act } from '@testing-library/react-native';
import * as SecureStore from 'expo-secure-store';

// Must mock before importing the store
jest.mock('expo-secure-store', () => ({
  getItemAsync: jest.fn(),
  setItemAsync: jest.fn(),
  deleteItemAsync: jest.fn(),
}));

jest.mock('@/src/api/auth', () => ({
  login: jest.fn(),
  register: jest.fn(),
  refresh: jest.fn(),
  logout: jest.fn(),
}));

jest.mock('@/src/api/client', () => ({
  setTokenGetter: jest.fn(),
  setRefreshHandlers: jest.fn(),
}));

jest.mock('@/src/utils/notifications', () => ({
  registerForPushNotifications: jest.fn().mockResolvedValue(undefined),
  unregisterPushNotifications: jest.fn().mockResolvedValue(undefined),
}));

import { useAuthStore } from '@/src/stores/auth';
import * as authApi from '@/src/api/auth';
import {
  registerForPushNotifications,
  unregisterPushNotifications,
} from '@/src/utils/notifications';

const mockLogin = authApi.login as jest.MockedFunction<typeof authApi.login>;
const mockRegister = authApi.register as jest.MockedFunction<typeof authApi.register>;
const mockRefresh = authApi.refresh as jest.MockedFunction<typeof authApi.refresh>;
const mockLogout = authApi.logout as jest.MockedFunction<typeof authApi.logout>;
const mockGetItem = SecureStore.getItemAsync as jest.MockedFunction<typeof SecureStore.getItemAsync>;
const mockSetItem = SecureStore.setItemAsync as jest.MockedFunction<typeof SecureStore.setItemAsync>;
const mockDeleteItem = SecureStore.deleteItemAsync as jest.MockedFunction<typeof SecureStore.deleteItemAsync>;
const mockRegisterPush = registerForPushNotifications as jest.MockedFunction<typeof registerForPushNotifications>;
const mockUnregisterPush = unregisterPushNotifications as jest.MockedFunction<typeof unregisterPushNotifications>;

const MOCK_USER = {
  id: 'user-1',
  email: 'test@example.com',
  display_name: 'Test',
  created_at: '2025-01-01T00:00:00Z',
};

const MOCK_AUTH_RESPONSE = {
  tokens: {
    access_token: 'access_123',
    refresh_token: 'refresh_123',
    token_type: 'Bearer',
    expires_in: 900,
  },
  user: MOCK_USER,
};

function resetStore() {
  useAuthStore.setState({
    accessToken: null,
    user: null,
    isAuthenticated: false,
    isLoading: true,
  });
}

beforeEach(() => {
  jest.clearAllMocks();
  resetStore();
});

describe('useAuthStore', () => {
  describe('login', () => {
    it('stores tokens and user on success', async () => {
      mockLogin.mockResolvedValueOnce(MOCK_AUTH_RESPONSE);

      await act(async () => {
        await useAuthStore.getState().login('test@example.com', 'password123');
      });

      const state = useAuthStore.getState();
      expect(state.accessToken).toBe('access_123');
      expect(state.user?.email).toBe('test@example.com');
      expect(state.isAuthenticated).toBe(true);
      expect(mockSetItem).toHaveBeenCalledWith('offrii_refresh_token', 'refresh_123');
      expect(mockSetItem).toHaveBeenCalledWith('offrii_user_data', JSON.stringify(MOCK_USER));
      expect(mockRegisterPush).toHaveBeenCalled();
    });

    it('propagates API errors', async () => {
      mockLogin.mockRejectedValueOnce(new Error('invalid credentials'));

      await expect(
        useAuthStore.getState().login('test@example.com', 'wrong'),
      ).rejects.toThrow('invalid credentials');

      expect(useAuthStore.getState().isAuthenticated).toBe(false);
    });
  });

  describe('register', () => {
    it('stores tokens and user on success', async () => {
      mockRegister.mockResolvedValueOnce(MOCK_AUTH_RESPONSE);

      await act(async () => {
        await useAuthStore.getState().register('test@example.com', 'password123', 'Test');
      });

      const state = useAuthStore.getState();
      expect(state.accessToken).toBe('access_123');
      expect(state.isAuthenticated).toBe(true);
      expect(mockRegister).toHaveBeenCalledWith('test@example.com', 'password123', 'Test');
      expect(mockSetItem).toHaveBeenCalledWith('offrii_user_data', JSON.stringify(MOCK_USER));
      expect(mockRegisterPush).toHaveBeenCalled();
    });
  });

  describe('logout', () => {
    it('calls API before clearing state and SecureStore', async () => {
      useAuthStore.setState({
        accessToken: 'access_123',
        user: MOCK_USER,
        isAuthenticated: true,
      });
      mockLogout.mockResolvedValueOnce(undefined);

      await act(async () => {
        await useAuthStore.getState().logout();
      });

      // Push token should be unregistered before API logout
      expect(mockUnregisterPush).toHaveBeenCalled();
      expect(mockLogout).toHaveBeenCalled();

      const state = useAuthStore.getState();
      expect(state.accessToken).toBeNull();
      expect(state.user).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      expect(mockDeleteItem).toHaveBeenCalledWith('offrii_refresh_token');
      expect(mockDeleteItem).toHaveBeenCalledWith('offrii_user_data');
    });

    it('clears state even if push unregister fails', async () => {
      useAuthStore.setState({
        accessToken: 'access_123',
        user: MOCK_USER,
        isAuthenticated: true,
      });
      mockUnregisterPush.mockRejectedValueOnce(new Error('SecureStore error'));
      mockLogout.mockResolvedValueOnce(undefined);

      await act(async () => {
        await useAuthStore.getState().logout();
      });

      const state = useAuthStore.getState();
      expect(state.accessToken).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      expect(mockLogout).toHaveBeenCalled();
    });

    it('clears state even if API call fails', async () => {
      useAuthStore.setState({
        accessToken: 'access_123',
        user: MOCK_USER,
        isAuthenticated: true,
      });
      mockLogout.mockRejectedValueOnce(new Error('network error'));

      await act(async () => {
        await useAuthStore.getState().logout();
      });

      const state = useAuthStore.getState();
      expect(state.accessToken).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      expect(mockDeleteItem).toHaveBeenCalledWith('offrii_refresh_token');
    });
  });

  describe('restoreSession', () => {
    it('refreshes token and restores cached user', async () => {
      mockGetItem
        .mockResolvedValueOnce('old_refresh')
        .mockResolvedValueOnce(JSON.stringify(MOCK_USER));
      mockRefresh.mockResolvedValueOnce({
        tokens: {
          access_token: 'new_access',
          refresh_token: 'new_refresh',
          token_type: 'Bearer',
          expires_in: 900,
        },
      });

      await act(async () => {
        await useAuthStore.getState().restoreSession();
      });

      const state = useAuthStore.getState();
      expect(state.accessToken).toBe('new_access');
      expect(state.user).toEqual(MOCK_USER);
      expect(state.isAuthenticated).toBe(true);
      expect(state.isLoading).toBe(false);
      expect(mockSetItem).toHaveBeenCalledWith('offrii_refresh_token', 'new_refresh');
      expect(mockRegisterPush).toHaveBeenCalled();
    });

    it('stays unauthenticated when no stored token', async () => {
      mockGetItem.mockResolvedValueOnce(null);

      await act(async () => {
        await useAuthStore.getState().restoreSession();
      });

      const state = useAuthStore.getState();
      expect(state.isAuthenticated).toBe(false);
      expect(state.isLoading).toBe(false);
    });

    it('clears tokens on refresh failure', async () => {
      mockGetItem.mockResolvedValueOnce('expired_refresh');
      mockRefresh.mockRejectedValueOnce(new Error('token expired'));

      await act(async () => {
        await useAuthStore.getState().restoreSession();
      });

      const state = useAuthStore.getState();
      expect(state.isAuthenticated).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(mockDeleteItem).toHaveBeenCalledWith('offrii_refresh_token');
      expect(mockDeleteItem).toHaveBeenCalledWith('offrii_user_data');
    });
  });
});
