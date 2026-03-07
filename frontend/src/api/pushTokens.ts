import { apiClient } from './client';

export async function registerPushToken(token: string, platform: string): Promise<void> {
  await apiClient.post('/push-tokens', { token, platform });
}

export async function unregisterPushToken(token: string): Promise<void> {
  await apiClient.delete(`/push-tokens/${encodeURIComponent(token)}`);
}
