import { render, screen } from '@testing-library/react-native';
import { PaperProvider } from 'react-native-paper';
import { paperTheme } from '@/src/theme';
import { EmptyState } from '@/src/components/items/EmptyState';

function renderComponent(status: 'active' | 'purchased') {
  return render(
    <PaperProvider theme={paperTheme}>
      <EmptyState status={status} />
    </PaperProvider>,
  );
}

describe('EmptyState', () => {
  it('renders active state with subtitle', () => {
    renderComponent('active');
    expect(screen.getByTestId('empty-state')).toBeTruthy();
    expect(screen.getByText('Aucun article')).toBeTruthy();
    expect(screen.getByText('Capture ta première offre !')).toBeTruthy();
  });

  it('renders purchased state without subtitle', () => {
    renderComponent('purchased');
    expect(screen.getByTestId('empty-state')).toBeTruthy();
    expect(screen.getByText('Aucun article acheté')).toBeTruthy();
    expect(screen.queryByText('Capture ta première offre !')).toBeNull();
  });
});
