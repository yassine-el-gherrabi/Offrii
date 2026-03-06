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

  it('shows weak for short passwords (< 8 chars, no diversity)', () => {
    renderIndicator('short');
    expect(screen.getByTestId('strength-bar')).toBeTruthy();
    expect(screen.getByText('Faible')).toBeTruthy();
  });

  it('shows weak for 8+ chars with no character diversity (score=1)', () => {
    renderIndicator('abcdefgh'); // length>=8 but no uppercase, digit, symbol
    expect(screen.getByText('Faible')).toBeTruthy();
  });

  it('shows good for 8+ chars with some diversity (score 2-3)', () => {
    renderIndicator('abcdefG1'); // length>=8 + uppercase + digit = score 3
    expect(screen.getByText('Correct')).toBeTruthy();
  });

  it('shows good for 12+ lowercase-only (score=2)', () => {
    renderIndicator('abcdefghijkl'); // length>=8 + length>=12 = score 2
    expect(screen.getByText('Correct')).toBeTruthy();
  });

  it('shows strong for 12+ chars with full diversity (score 4+)', () => {
    renderIndicator('Abcdefghijk1!'); // length>=8 + length>=12 + upper + digit + symbol = score 5
    expect(screen.getByText('Fort')).toBeTruthy();
  });

  it('shows strong for shorter password with all diversity types (score 4)', () => {
    renderIndicator('Abcdef1!'); // length>=8 + upper + digit + symbol = score 4
    expect(screen.getByText('Fort')).toBeTruthy();
  });
});
