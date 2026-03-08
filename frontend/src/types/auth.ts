export interface TokenPair {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface UserResponse {
  id: string;
  email: string;
  display_name: string | null;
  created_at: string;
}

export interface AuthResponse {
  tokens: TokenPair;
  user: UserResponse;
}

export interface RefreshResponse {
  tokens: TokenPair;
}

export interface UserProfileResponse {
  id: string;
  email: string;
  display_name: string | null;
  reminder_freq: 'never' | 'daily' | 'weekly' | 'monthly';
  reminder_time: string;
  timezone: string;
  locale: string;
  created_at: string;
}

export interface UpdateProfileRequest {
  display_name?: string;
  reminder_freq?: string;
  reminder_time?: string;
  timezone?: string;
  locale?: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface UserDataExport {
  profile: UserProfileResponse;
  items: unknown[];
  categories: unknown[];
  exported_at: string;
}

export interface ForgotPasswordRequest {
  email: string;
}

export interface ResetPasswordRequest {
  email: string;
  code: string;
  new_password: string;
}

export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}
