import axios from 'axios';
import MockAdapter from 'axios-mock-adapter';

jest.mock('@/src/constants/api', () => ({
  API_BASE_URL: 'http://test-api.local',
}));

import { apiClient, setTokenGetter, ApiRequestError } from '@/src/api/client';

const mock = new MockAdapter(apiClient);

beforeEach(() => {
  mock.reset();
  setTokenGetter(null as unknown as () => string | null);
});

describe('apiClient', () => {
  describe('request interceptor', () => {
    it('attaches Authorization header when token is available', async () => {
      setTokenGetter(() => 'test-token-123');
      mock.onGet('/test').reply(200, { ok: true });

      await apiClient.get('/test');

      expect(mock.history.get[0]?.headers?.Authorization).toBe('Bearer test-token-123');
    });

    it('does not attach Authorization header when no token', async () => {
      setTokenGetter(() => null);
      mock.onGet('/test').reply(200, { ok: true });

      await apiClient.get('/test');

      expect(mock.history.get[0]?.headers?.Authorization).toBeUndefined();
    });
  });

  describe('response interceptor', () => {
    it('passes through successful responses', async () => {
      mock.onGet('/test').reply(200, { data: 'hello' });

      const response = await apiClient.get('/test');
      expect(response.data).toEqual({ data: 'hello' });
    });

    it('extracts error message from API error format', async () => {
      mock.onPost('/auth/login').reply(401, {
        error: { code: 'INVALID_CREDENTIALS', message: 'Bad password' },
      });

      await expect(apiClient.post('/auth/login')).rejects.toThrow(ApiRequestError);
      try {
        await apiClient.post('/auth/login');
      } catch (e) {
        expect(e).toBeInstanceOf(ApiRequestError);
        expect((e as ApiRequestError).message).toBe('Bad password');
        expect((e as ApiRequestError).status).toBe(401);
      }
    });

    it('returns network error with status 0 when no response', async () => {
      mock.onGet('/test').networkError();

      try {
        await apiClient.get('/test');
        fail('should have thrown');
      } catch (e) {
        expect(e).toBeInstanceOf(ApiRequestError);
        expect((e as ApiRequestError).status).toBe(0);
        expect((e as ApiRequestError).message).toBe('Network error');
      }
    });

    it('returns status code for server errors', async () => {
      mock.onGet('/test').reply(500, {
        error: { code: 'INTERNAL', message: 'Server broke' },
      });

      try {
        await apiClient.get('/test');
        fail('should have thrown');
      } catch (e) {
        expect(e).toBeInstanceOf(ApiRequestError);
        expect((e as ApiRequestError).status).toBe(500);
        expect((e as ApiRequestError).message).toBe('Server broke');
      }
    });
  });
});
