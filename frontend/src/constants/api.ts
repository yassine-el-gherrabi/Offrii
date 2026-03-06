import { Platform } from 'react-native';
import Constants from 'expo-constants';

function getDevBaseUrl(): string {
  // In dev, all traffic goes through Caddy (reverse proxy with CORS headers)
  const host = Platform.OS === 'web'
    ? 'localhost'
    : (Constants.expoConfig?.hostUri?.split(':')[0] ?? 'localhost');
  return `http://${host}:80`;
}

export const API_BASE_URL = __DEV__ ? getDevBaseUrl() : 'https://api.offrii.com';
