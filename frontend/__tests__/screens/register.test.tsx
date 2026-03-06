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
const mockRegister = jest.fn();
jest.mock('@/src/stores/auth', () => ({
  useAuthStore: (selector: (s: Record<string, unknown>) => unknown) =>
    selector({ register: mockRegister }),
}));

import RegisterScreen from '@/app/(auth)/register';
import { ApiRequestError } from '@/src/api/client';

function renderRegister() {
  return render(
    <PaperProvider theme={paperTheme}>
      <RegisterScreen />
    </PaperProvider>,
  );
}

beforeEach(() => {
  jest.clearAllMocks();
});

describe('RegisterScreen', () => {
  it('renders all 3 fields', () => {
    renderRegister();
    expect(screen.getByTestId('email-input')).toBeTruthy();
    expect(screen.getByTestId('password-input')).toBeTruthy();
    expect(screen.getByTestId('displayname-input')).toBeTruthy();
    expect(screen.getByTestId('register-button')).toBeTruthy();
  });

  it('shows error when password is less than 8 chars', async () => {
    renderRegister();
    fireEvent.changeText(screen.getByTestId('email-input'), 'test@example.com');
    fireEvent.changeText(screen.getByTestId('password-input'), 'short');
    fireEvent.press(screen.getByTestId('register-button'));

    await waitFor(() => {
      expect(screen.getByText('8 caractères minimum')).toBeTruthy();
    });
  });

  it('shows error for invalid email format', async () => {
    renderRegister();
    fireEvent.changeText(screen.getByTestId('email-input'), 'bademail');
    fireEvent.changeText(screen.getByTestId('password-input'), 'password123');
    fireEvent.press(screen.getByTestId('register-button'));

    await waitFor(() => {
      expect(screen.getByText("Format d'email invalide")).toBeTruthy();
    });
  });

  it('shows inline error on 409 (email taken)', async () => {
    mockRegister.mockRejectedValueOnce(new ApiRequestError('email taken', 409));
    renderRegister();

    fireEvent.changeText(screen.getByTestId('email-input'), 'taken@example.com');
    fireEvent.changeText(screen.getByTestId('password-input'), 'password123');
    fireEvent.press(screen.getByTestId('register-button'));

    await waitFor(() => {
      expect(screen.getByText('Cet email est déjà utilisé')).toBeTruthy();
    });
  });

  it('calls register and navigates on success', async () => {
    mockRegister.mockResolvedValueOnce(undefined);
    renderRegister();

    fireEvent.changeText(screen.getByTestId('email-input'), 'new@example.com');
    fireEvent.changeText(screen.getByTestId('password-input'), 'password123');
    fireEvent.changeText(screen.getByTestId('displayname-input'), 'Alice');
    fireEvent.press(screen.getByTestId('register-button'));

    await waitFor(() => {
      expect(mockRegister).toHaveBeenCalledWith('new@example.com', 'password123', 'Alice');
      expect(mockReplace).toHaveBeenCalledWith('/(tabs)/capture');
    });
  });

  it('navigates to login screen', () => {
    renderRegister();
    fireEvent.press(screen.getByTestId('goto-login'));
    expect(mockPush).toHaveBeenCalledWith('/(auth)/login');
  });
});
