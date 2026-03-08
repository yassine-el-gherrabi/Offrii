import { apiClient } from './client';
import type {
  ChangePasswordRequest,
  UpdateProfileRequest,
  UserDataExport,
  UserProfileResponse,
} from '@/src/types/auth';

export async function getProfile(): Promise<UserProfileResponse> {
  const { data } = await apiClient.get<UserProfileResponse>('/users/me');
  return data;
}

export async function updateProfile(
  req: UpdateProfileRequest,
): Promise<UserProfileResponse> {
  const { data } = await apiClient.patch<UserProfileResponse>('/users/me', req);
  return data;
}

export async function deleteAccount(): Promise<void> {
  await apiClient.delete('/users/me');
}

export async function changePassword(req: ChangePasswordRequest): Promise<void> {
  await apiClient.post('/auth/change-password', req);
}

export async function exportData(): Promise<UserDataExport> {
  const { data } = await apiClient.get<UserDataExport>('/users/me/export');
  return data;
}
