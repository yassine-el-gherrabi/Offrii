import { render, screen, fireEvent, waitFor } from '@testing-library/react-native';
import { PaperProvider } from 'react-native-paper';
import { paperTheme } from '@/src/theme';

// Mock expo-router
const mockReplace = jest.fn();
const mockPush = jest.fn();
jest.mock('expo-router', () => ({
  router: {
    replace: (...args: unknown[]) => mockReplace(...args),
    push: (...args: unknown[]) => mockPush(...args),
  },
}));

// Mock auth store
const mockLogin = jest.fn();
jest.mock('@/src/stores/auth', () => ({
  useAuthStore: (selector: (s: Record<string, unknown>) => unknown) =>
    selector({ login: mockLogin }),
}));

import LoginScreen from '@/app/(auth)/login';
import { ApiRequestError } from '@/src/api/client';

function renderLogin() {
  return render(
    <PaperProvider theme={paperTheme}>
      <LoginScreen />
    </PaperProvider>,
  );
}

beforeEach(() => {
  jest.clearAllMocks();
});

describe('LoginScreen', () => {
  it('renders the form fields', () => {
    renderLogin();
    expect(screen.getByTestId('email-input')).toBeTruthy();
    expect(screen.getByTestId('password-input')).toBeTruthy();
    expect(screen.getByTestId('login-button')).toBeTruthy();
  });

  it('shows validation errors when submitting empty fields', async () => {
    renderLogin();
    fireEvent.press(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(screen.getByTestId('email-error')).toBeTruthy();
      expect(screen.getByTestId('password-error')).toBeTruthy();
    });
  });

  it('shows email format error for invalid email', async () => {
    renderLogin();
    fireEvent.changeText(screen.getByTestId('email-input'), 'notanemail');
    fireEvent.press(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(screen.getByText("Format d'email invalide")).toBeTruthy();
    });
  });

  it('calls login and navigates on success', async () => {
    mockLogin.mockResolvedValueOnce(undefined);
    renderLogin();

    fireEvent.changeText(screen.getByTestId('email-input'), 'test@example.com');
    fireEvent.changeText(screen.getByTestId('password-input'), 'password123');
    fireEvent.press(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(mockLogin).toHaveBeenCalledWith('test@example.com', 'password123');
      expect(mockReplace).toHaveBeenCalledWith('/(tabs)/capture');
    });
  });

  it('shows API error on 401', async () => {
    mockLogin.mockRejectedValueOnce(new ApiRequestError('unauthorized', 401));
    renderLogin();

    fireEvent.changeText(screen.getByTestId('email-input'), 'test@example.com');
    fireEvent.changeText(screen.getByTestId('password-input'), 'wrongpassword');
    fireEvent.press(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(screen.getByText('Email ou mot de passe incorrect')).toBeTruthy();
    });
  });

  it('navigates to register screen', () => {
    renderLogin();
    fireEvent.press(screen.getByTestId('goto-register'));
    expect(mockPush).toHaveBeenCalledWith('/(auth)/register');
  });
});
