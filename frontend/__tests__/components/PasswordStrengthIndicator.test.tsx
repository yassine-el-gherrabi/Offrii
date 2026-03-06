import { render, screen } from '@testing-library/react-native';
import { PaperProvider } from 'react-native-paper';
import { paperTheme } from '@/src/theme';
import PasswordStrengthIndicator from '@/src/components/PasswordStrengthIndicator';

function renderIndicator(password: string) {
  return render(
    <PaperProvider theme={paperTheme}>
      <PasswordStrengthIndicator password={password} />
    </PaperProvider>,
  );
}

describe('PasswordStrengthIndicator', () => {
  it('renders nothing when password is empty', () => {
    renderIndicator('');
    expect(screen.queryByTestId('strength-bar')).toBeNull();
    expect(screen.queryByTestId('strength-label')).toBeNull();
  });

  it('shows weak (red) for < 8 chars', () => {
    renderIndicator('short');
    expect(screen.getByTestId('strength-bar')).toBeTruthy();
    expect(screen.getByText('Faible')).toBeTruthy();
  });

  it('shows good (orange) for 8-11 chars', () => {
    renderIndicator('abcdefgh'); // 8 chars
    expect(screen.getByTestId('strength-bar')).toBeTruthy();
    expect(screen.getByText('Correct')).toBeTruthy();
  });

  it('shows strong (green) for 12+ chars', () => {
    renderIndicator('abcdefghijkl'); // 12 chars
    expect(screen.getByTestId('strength-bar')).toBeTruthy();
    expect(screen.getByText('Fort')).toBeTruthy();
  });
});
