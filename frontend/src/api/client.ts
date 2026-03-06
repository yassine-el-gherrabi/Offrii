import axios from 'axios';
import type { AxiosError, InternalAxiosRequestConfig } from 'axios';
import { API_BASE_URL } from '@/src/constants/api';
import type { ApiError } from '@/src/types/auth';

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 15_000,
  headers: { 'Content-Type': 'application/json' },
});

/**
 * Attach the access token to outgoing requests.
 * Called by the auth store once initialized to avoid circular imports.
 */
let getAccessToken: (() => string | null) | null = null;

export function setTokenGetter(getter: () => string | null) {
  getAccessToken = getter;
}

/**
 * Handlers injected by the auth store for token refresh.
 * Avoids circular imports between client and auth modules.
 */
interface RefreshHandlers {
  getRefreshToken: () => Promise<string | null>;
  onRefreshSuccess: (accessToken: string, refreshToken: string) => Promise<void>;
  onRefreshFailure: () => void;
}

let refreshHandlers: RefreshHandlers | null = null;

export function setRefreshHandlers(handlers: RefreshHandlers) {
  refreshHandlers = handlers;
}

// --- Request interceptor ---

apiClient.interceptors.request.use((config) => {
  const token = getAccessToken?.();
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// --- Response interceptor with 401 refresh logic ---

let isRefreshing = false;
let failedQueue: {
  resolve: (token: string) => void;
  reject: (error: unknown) => void;
}[] = [];

function processQueue(error: unknown, token: string | null) {
  failedQueue.forEach(({ resolve, reject }) => {
    if (error || !token) {
      reject(error);
    } else {
      resolve(token);
    }
  });
  failedQueue = [];
}

apiClient.interceptors.response.use(
  (response) => response,
  async (error: AxiosError<ApiError>) => {
    const originalRequest = error.config as InternalAxiosRequestConfig & { _retry?: boolean };

    // Only attempt refresh on 401 when handlers are set and request hasn't been retried
    if (
      error.response?.status === 401 &&
      refreshHandlers &&
      originalRequest &&
      !originalRequest._retry &&
      !originalRequest.url?.includes('/auth/refresh') &&
      !originalRequest.url?.includes('/auth/login') &&
      !originalRequest.url?.includes('/auth/register')
    ) {
      if (isRefreshing) {
        // Queue this request until the ongoing refresh completes
        return new Promise<string>((resolve, reject) => {
          failedQueue.push({ resolve, reject });
        }).then((token) => {
          originalRequest.headers.Authorization = `Bearer ${token}`;
          return apiClient(originalRequest);
        });
      }

      originalRequest._retry = true;
      isRefreshing = true;

      try {
        const refreshToken = await refreshHandlers.getRefreshToken();
        if (!refreshToken) {
          throw new Error('No refresh token');
        }

        // Call refresh endpoint directly to avoid interceptor loop
        const { data } = await apiClient.post('/auth/refresh', {
          refresh_token: refreshToken,
        });

        const newAccessToken = data.tokens.access_token;
        const newRefreshToken = data.tokens.refresh_token;

        await refreshHandlers.onRefreshSuccess(newAccessToken, newRefreshToken);
        processQueue(null, newAccessToken);

        // Retry the original request with the new token
        originalRequest.headers.Authorization = `Bearer ${newAccessToken}`;
        return apiClient(originalRequest);
      } catch (refreshError) {
        processQueue(refreshError, null);
        refreshHandlers.onRefreshFailure();
        return Promise.reject(new ApiRequestError('Session expired', 401));
      } finally {
        isRefreshing = false;
      }
    }

    if (!error.response) {
      // Network error (timeout, DNS, ECONNREFUSED) — no HTTP response received
      return Promise.reject(new ApiRequestError('Network error', 0));
    }
    const message =
      error.response.data?.error?.message ?? error.message ?? 'An unexpected error occurred';
    return Promise.reject(new ApiRequestError(message, error.response.status));
  },
);

export class ApiRequestError extends Error {
  constructor(
    message: string,
    public readonly status?: number,
  ) {
    super(message);
    this.name = 'ApiRequestError';
  }
}
