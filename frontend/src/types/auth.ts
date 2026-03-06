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

export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}
