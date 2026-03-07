import { render, screen, fireEvent } from '@testing-library/react-native';
import { PaperProvider } from 'react-native-paper';
import { paperTheme } from '@/src/theme';
import { CategoryChip } from '@/src/components/items/CategoryChip';
import type { CategoryResponse } from '@/src/types/items';

const MOCK_CATEGORY: CategoryResponse = {
  id: 'cat-1',
  name: 'Tech',
  icon: 'laptop',
  is_default: false,
  position: 0,
  created_at: '2025-01-01T00:00:00Z',
};

function renderComponent(props: Partial<React.ComponentProps<typeof CategoryChip>> = {}) {
  return render(
    <PaperProvider theme={paperTheme}>
      <CategoryChip category={MOCK_CATEGORY} {...props} />
    </PaperProvider>,
  );
}

describe('CategoryChip', () => {
  it('renders category name', () => {
    renderComponent();
    expect(screen.getByText('Tech')).toBeTruthy();
  });

  it('calls onPress when pressed', () => {
    const onPress = jest.fn();
    renderComponent({ onPress });
    fireEvent.press(screen.getByTestId('category-chip-cat-1'));
    expect(onPress).toHaveBeenCalledTimes(1);
  });

  it('renders with testID containing category id', () => {
    renderComponent();
    expect(screen.getByTestId('category-chip-cat-1')).toBeTruthy();
  });
});
