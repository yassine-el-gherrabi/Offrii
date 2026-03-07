import { render, screen } from '@testing-library/react-native';
import { PaperProvider } from 'react-native-paper';
import { paperTheme } from '@/src/theme';
import { AgeBadge } from '@/src/components/items/AgeBadge';

function renderComponent(props: { createdAt: string; size?: 'small' | 'medium' }) {
  return render(
    <PaperProvider theme={paperTheme}>
      <AgeBadge {...props} />
    </PaperProvider>,
  );
}

describe('AgeBadge', () => {
  it('renders the days count', () => {
    const today = new Date().toISOString();
    renderComponent({ createdAt: today });
    expect(screen.getByTestId('age-badge-text')).toBeTruthy();
    expect(screen.getByText('0j')).toBeTruthy();
  });

  it('shows subtitle in medium size', () => {
    const today = new Date().toISOString();
    renderComponent({ createdAt: today, size: 'medium' });
    expect(screen.getByTestId('age-badge-subtitle')).toBeTruthy();
  });

  it('does not show subtitle in small size', () => {
    const today = new Date().toISOString();
    renderComponent({ createdAt: today, size: 'small' });
    expect(screen.queryByTestId('age-badge-subtitle')).toBeNull();
  });
});
