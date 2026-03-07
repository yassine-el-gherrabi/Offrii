import * as Notifications from 'expo-notifications';
import * as SecureStore from 'expo-secure-store';
import { Platform } from 'react-native';

jest.mock('expo-notifications', () => ({
  getPermissionsAsync: jest.fn(),
  requestPermissionsAsync: jest.fn(),
  getExpoPushTokenAsync: jest.fn(),
  setNotificationChannelAsync: jest.fn(),
  setNotificationHandler: jest.fn(),
  addNotificationResponseReceivedListener: jest.fn(),
  getLastNotificationResponseAsync: jest.fn(),
  AndroidImportance: { HIGH: 4 },
  PermissionStatus: { GRANTED: 'granted', DENIED: 'denied', UNDETERMINED: 'undetermined' },
}));

let mockIsDevice = true;
jest.mock('expo-device', () => ({
  get isDevice() {
    return mockIsDevice;
  },
}));

jest.mock('expo-secure-store', () => ({
  getItemAsync: jest.fn(),
  setItemAsync: jest.fn(),
  deleteItemAsync: jest.fn(),
}));

jest.mock('expo-constants', () => ({
  expoConfig: { extra: { eas: { projectId: 'test-project-id' } } },
}));

jest.mock('@/src/api/pushTokens', () => ({
  registerPushToken: jest.fn().mockResolvedValue(undefined),
  unregisterPushToken: jest.fn().mockResolvedValue(undefined),
}));

jest.mock('@/src/constants/routes', () => ({
  ROUTES: { ITEM_LIST: '/(tabs)/list' },
}));

import {
  registerForPushNotifications,
  unregisterPushNotifications,
  setupNotificationHandler,
  setupNotificationResponseListener,
  handleInitialNotification,
} from '@/src/utils/notifications';
import * as pushTokensApi from '@/src/api/pushTokens';

const mockGetPermissions = Notifications.getPermissionsAsync as jest.MockedFunction<
  typeof Notifications.getPermissionsAsync
>;
const mockRequestPermissions = Notifications.requestPermissionsAsync as jest.MockedFunction<
  typeof Notifications.requestPermissionsAsync
>;
const mockGetExpoPushToken = Notifications.getExpoPushTokenAsync as jest.MockedFunction<
  typeof Notifications.getExpoPushTokenAsync
>;
const mockSetChannel = Notifications.setNotificationChannelAsync as jest.MockedFunction<
  typeof Notifications.setNotificationChannelAsync
>;
const mockSetHandler = Notifications.setNotificationHandler as jest.MockedFunction<
  typeof Notifications.setNotificationHandler
>;
const mockAddResponseListener =
  Notifications.addNotificationResponseReceivedListener as jest.MockedFunction<
    typeof Notifications.addNotificationResponseReceivedListener
  >;
const mockGetLastResponse = Notifications.getLastNotificationResponseAsync as jest.MockedFunction<
  typeof Notifications.getLastNotificationResponseAsync
>;
const mockRegisterToken = pushTokensApi.registerPushToken as jest.MockedFunction<
  typeof pushTokensApi.registerPushToken
>;
const mockUnregisterToken = pushTokensApi.unregisterPushToken as jest.MockedFunction<
  typeof pushTokensApi.unregisterPushToken
>;
const mockGetItem = SecureStore.getItemAsync as jest.MockedFunction<
  typeof SecureStore.getItemAsync
>;
const mockSetItem = SecureStore.setItemAsync as jest.MockedFunction<
  typeof SecureStore.setItemAsync
>;
const mockDeleteItem = SecureStore.deleteItemAsync as jest.MockedFunction<
  typeof SecureStore.deleteItemAsync
>;

beforeEach(() => {
  jest.clearAllMocks();
});

describe('registerForPushNotifications', () => {
  it('skips on non-device (simulator)', async () => {
    mockIsDevice = false;

    await registerForPushNotifications();

    expect(mockGetPermissions).not.toHaveBeenCalled();

    mockIsDevice = true;
  });

  it('requests permission when not already granted', async () => {
    mockGetPermissions.mockResolvedValueOnce({
      status: Notifications.PermissionStatus.UNDETERMINED,
      expires: 'never',
      granted: false,
      canAskAgain: true,
    });
    mockRequestPermissions.mockResolvedValueOnce({
      status: Notifications.PermissionStatus.GRANTED,
      expires: 'never',
      granted: true,
      canAskAgain: true,
    });
    mockGetExpoPushToken.mockResolvedValueOnce({
      data: 'ExponentPushToken[xxx]',
      type: 'expo',
    });

    await registerForPushNotifications();

    expect(mockRequestPermissions).toHaveBeenCalled();
    expect(mockGetExpoPushToken).toHaveBeenCalledWith({ projectId: 'test-project-id' });
    expect(mockRegisterToken).toHaveBeenCalledWith('ExponentPushToken[xxx]', Platform.OS);
    expect(mockSetItem).toHaveBeenCalledWith('offrii_push_token', 'ExponentPushToken[xxx]');
  });

  it('skips when permission is denied', async () => {
    mockGetPermissions.mockResolvedValueOnce({
      status: Notifications.PermissionStatus.UNDETERMINED,
      expires: 'never',
      granted: false,
      canAskAgain: true,
    });
    mockRequestPermissions.mockResolvedValueOnce({
      status: Notifications.PermissionStatus.DENIED,
      expires: 'never',
      granted: false,
      canAskAgain: false,
    });

    await registerForPushNotifications();

    expect(mockGetExpoPushToken).not.toHaveBeenCalled();
    expect(mockRegisterToken).not.toHaveBeenCalled();
  });

  it('skips permission request when already granted', async () => {
    mockGetPermissions.mockResolvedValueOnce({
      status: Notifications.PermissionStatus.GRANTED,
      expires: 'never',
      granted: true,
      canAskAgain: true,
    });
    mockGetExpoPushToken.mockResolvedValueOnce({
      data: 'ExponentPushToken[yyy]',
      type: 'expo',
    });

    await registerForPushNotifications();

    expect(mockRequestPermissions).not.toHaveBeenCalled();
    expect(mockRegisterToken).toHaveBeenCalledWith('ExponentPushToken[yyy]', Platform.OS);
  });

  it('creates Android notification channel on Android', async () => {
    const originalSelect = Platform.select;
    const originalOS = Platform.OS;
    (Platform as { OS: string }).OS = 'android';
    Platform.select = (obj: Record<string, unknown>) => obj.android ?? obj.default;

    mockGetPermissions.mockResolvedValueOnce({
      status: Notifications.PermissionStatus.GRANTED,
      expires: 'never',
      granted: true,
      canAskAgain: true,
    });
    mockGetExpoPushToken.mockResolvedValueOnce({
      data: 'ExponentPushToken[zzz]',
      type: 'expo',
    });

    await registerForPushNotifications();

    expect(mockSetChannel).toHaveBeenCalledWith('reminders', {
      name: 'Reminders',
      importance: Notifications.AndroidImportance.HIGH,
      sound: 'default',
    });

    (Platform as { OS: string }).OS = originalOS;
    Platform.select = originalSelect;
  });
});

describe('unregisterPushNotifications', () => {
  it('calls API and clears secure store when token exists', async () => {
    mockGetItem.mockResolvedValueOnce('ExponentPushToken[xxx]');

    await unregisterPushNotifications();

    expect(mockUnregisterToken).toHaveBeenCalledWith('ExponentPushToken[xxx]');
    expect(mockDeleteItem).toHaveBeenCalledWith('offrii_push_token');
  });

  it('does nothing when no stored token', async () => {
    mockGetItem.mockResolvedValueOnce(null);

    await unregisterPushNotifications();

    expect(mockUnregisterToken).not.toHaveBeenCalled();
    expect(mockDeleteItem).not.toHaveBeenCalled();
  });

  it('still clears secure store when API call fails', async () => {
    mockGetItem.mockResolvedValueOnce('ExponentPushToken[xxx]');
    mockUnregisterToken.mockRejectedValueOnce(new Error('network'));

    await unregisterPushNotifications();

    expect(mockDeleteItem).toHaveBeenCalledWith('offrii_push_token');
  });
});

describe('setupNotificationHandler', () => {
  it('sets foreground handler to silent mode', () => {
    setupNotificationHandler();

    expect(mockSetHandler).toHaveBeenCalledWith({
      handleNotification: expect.any(Function),
    });
  });

  it('handler returns all flags as false', async () => {
    setupNotificationHandler();

    const arg = mockSetHandler.mock.calls[0]![0] as unknown as {
      handleNotification: () => Promise<Record<string, boolean>>;
    };
    const result = await arg.handleNotification();

    expect(result).toEqual({
      shouldShowAlert: false,
      shouldShowBanner: false,
      shouldShowList: false,
      shouldPlaySound: false,
      shouldSetBadge: false,
    });
  });
});

describe('setupNotificationResponseListener', () => {
  it('registers a response listener and navigates on tap', () => {
    const mockRemove = jest.fn();
    mockAddResponseListener.mockReturnValueOnce({ remove: mockRemove } as unknown as Notifications.EventSubscription);

    const mockRouter = { push: jest.fn() } as unknown as Parameters<
      typeof setupNotificationResponseListener
    >[0];

    const subscription = setupNotificationResponseListener(mockRouter);

    expect(mockAddResponseListener).toHaveBeenCalledWith(expect.any(Function));

    // Simulate a tap
    const callback = mockAddResponseListener.mock.calls[0]![0];
    callback({} as Notifications.NotificationResponse);

    expect(mockRouter.push).toHaveBeenCalledWith('/(tabs)/list');

    // Cleanup works
    subscription.remove();
    expect(mockRemove).toHaveBeenCalled();
  });
});

describe('handleInitialNotification', () => {
  it('navigates when app was launched from a notification', async () => {
    mockGetLastResponse.mockResolvedValueOnce({} as Notifications.NotificationResponse);

    const mockRouter = { push: jest.fn() } as unknown as Parameters<
      typeof handleInitialNotification
    >[0];

    await handleInitialNotification(mockRouter);

    expect(mockRouter.push).toHaveBeenCalledWith('/(tabs)/list');
  });

  it('does nothing when app was not launched from a notification', async () => {
    mockGetLastResponse.mockResolvedValueOnce(null);

    const mockRouter = { push: jest.fn() } as unknown as Parameters<
      typeof handleInitialNotification
    >[0];

    await handleInitialNotification(mockRouter);

    expect(mockRouter.push).not.toHaveBeenCalled();
  });
});
