import * as Notifications from 'expo-notifications';
import * as Device from 'expo-device';
import * as SecureStore from 'expo-secure-store';
import Constants from 'expo-constants';
import { Platform } from 'react-native';
import type { Router } from 'expo-router';

import * as pushTokensApi from '@/src/api/pushTokens';
import { ROUTES } from '@/src/constants/routes';

const PUSH_TOKEN_KEY = 'offrii_push_token';

/**
 * Request permission, get Expo push token, register it with the backend,
 * and create the Android notification channel.
 * Silently skips on simulators or when permission is denied.
 */
export async function registerForPushNotifications(): Promise<void> {
  if (!Device.isDevice) return;

  const { status: existing } = await Notifications.getPermissionsAsync();
  let finalStatus = existing;

  if (existing !== 'granted') {
    const { status } = await Notifications.requestPermissionsAsync();
    finalStatus = status;
  }

  if (finalStatus !== 'granted') return;

  const projectId = Constants.expoConfig?.extra?.eas?.projectId;
  if (!projectId) {
    console.warn('[notifications] Expo projectId is missing; skipping push token registration.');
    return;
  }

  const { data: token } = await Notifications.getExpoPushTokenAsync({
    projectId,
  });

  if (Platform.OS === 'android') {
    await Notifications.setNotificationChannelAsync('reminders', {
      name: 'Reminders',
      importance: Notifications.AndroidImportance.HIGH,
      sound: 'default',
    });
  }

  const storedToken = await SecureStore.getItemAsync(PUSH_TOKEN_KEY);
  if (storedToken === token) return;

  await pushTokensApi.registerPushToken(token, Platform.OS);
  await SecureStore.setItemAsync(PUSH_TOKEN_KEY, token);
}

/**
 * Unregister the stored push token from the backend (best-effort)
 * and remove it from secure storage.
 */
export async function unregisterPushNotifications(): Promise<void> {
  const token = await SecureStore.getItemAsync(PUSH_TOKEN_KEY);
  if (!token) return;

  try {
    await pushTokensApi.unregisterPushToken(token);
    await SecureStore.deleteItemAsync(PUSH_TOKEN_KEY);
  } catch {
    // Best-effort — keep local token for retry if backend call fails
  }
}

/**
 * Configure foreground notification behavior: silent (no alert, no sound, no badge).
 * Must be called at the module level, outside any component.
 */
export function setupNotificationHandler(): void {
  Notifications.setNotificationHandler({
    handleNotification: async () => ({
      shouldShowAlert: false,
      shouldShowBanner: false,
      shouldShowList: false,
      shouldPlaySound: false,
      shouldSetBadge: false,
    }),
  });
}

/**
 * Listen for notification taps and navigate to the items list.
 * Returns the subscription for cleanup in useEffect.
 */
export function setupNotificationResponseListener(
  router: Router,
): Notifications.EventSubscription {
  return Notifications.addNotificationResponseReceivedListener(() => {
    router.push(ROUTES.ITEM_LIST);
  });
}

/**
 * Handle the notification that launched the app from a killed state.
 * Must be called inside a useEffect since the response event fires
 * before the live listener is registered.
 */
export async function handleInitialNotification(router: Router): Promise<void> {
  const response = await Notifications.getLastNotificationResponseAsync();
  if (response) {
    router.push(ROUTES.ITEM_LIST);
  }
}
