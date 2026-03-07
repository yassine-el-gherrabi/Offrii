import MockAdapter from 'axios-mock-adapter';

jest.mock('@/src/constants/api', () => ({
  API_BASE_URL: 'http://test-api.local',
}));

import { apiClient } from '@/src/api/client';
import { registerPushToken, unregisterPushToken } from '@/src/api/pushTokens';

const mock = new MockAdapter(apiClient);

beforeEach(() => {
  mock.reset();
});

describe('pushTokens API', () => {
  describe('registerPushToken', () => {
    it('sends POST /push-tokens with token and platform', async () => {
      mock.onPost('/push-tokens').reply(201);

      await registerPushToken('ExponentPushToken[xxx]', 'ios');

      expect(mock.history.post).toHaveLength(1);
      expect(JSON.parse(mock.history.post[0]!.data as string)).toEqual({
        token: 'ExponentPushToken[xxx]',
        platform: 'ios',
      });
    });
  });

  describe('unregisterPushToken', () => {
    it('sends DELETE /push-tokens/:token', async () => {
      mock.onDelete('/push-tokens/ExponentPushToken%5Bxxx%5D').reply(204);

      await unregisterPushToken('ExponentPushToken[xxx]');

      expect(mock.history.delete).toHaveLength(1);
      expect(mock.history.delete[0]!.url).toBe('/push-tokens/ExponentPushToken%5Bxxx%5D');
    });
  });
});
