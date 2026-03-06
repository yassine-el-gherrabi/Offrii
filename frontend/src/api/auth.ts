import { apiClient } from './client';
import type { AuthResponse, RefreshResponse } from '@/src/types/auth';

export async function register(
  email: string,
  password: string,
  displayName?: string,
): Promise<AuthResponse> {
  const { data } = await apiClient.post<AuthResponse>('/auth/register', {
    email,
    password,
    display_name: displayName || undefined,
  });
  return data;
}

export async function login(email: string, password: string): Promise<AuthResponse> {
  const { data } = await apiClient.post<AuthResponse>('/auth/login', { email, password });
  return data;
}

export async function refresh(refreshToken: string): Promise<RefreshResponse> {
  const { data } = await apiClient.post<RefreshResponse>('/auth/refresh', {
    refresh_token: refreshToken,
  });
  return data;
}

export async function logout(accessToken: string): Promise<void> {
  await apiClient.post('/auth/logout', null, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}
