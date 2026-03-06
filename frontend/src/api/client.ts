import axios from 'axios';
import type { AxiosError } from 'axios';
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

apiClient.interceptors.request.use((config) => {
  const token = getAccessToken?.();
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError<ApiError>) => {
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
