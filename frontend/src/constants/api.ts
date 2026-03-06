import Constants from 'expo-constants';

function getDevBaseUrl(): string {
  const host = Constants.expoConfig?.hostUri?.split(':')[0] ?? 'localhost';
  return `http://${host}:80`;
}

export const API_BASE_URL = __DEV__ ? getDevBaseUrl() : 'https://api.offrii.com';
