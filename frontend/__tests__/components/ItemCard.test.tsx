import { render, screen, fireEvent } from '@testing-library/react-native';
import { PaperProvider } from 'react-native-paper';
import { paperTheme } from '@/src/theme';
import { ItemCard } from '@/src/components/items/ItemCard';
import type { ItemResponse, CategoryResponse } from '@/src/types/items';

const MOCK_ITEM: ItemResponse = {
  id: 'item-1',
  name: 'Nike Air Max',
  description: null,
  url: null,
  estimated_price: '150.00',
  priority: 2,
  category_id: 'cat-1',
  status: 'active',
  purchased_at: null,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

const MOCK_CATEGORIES: CategoryResponse[] = [
  {
    id: 'cat-1',
    name: 'Mode',
    icon: null,
    is_default: false,
    position: 0,
    created_at: '2025-01-01T00:00:00Z',
  },
];

function renderComponent(props: Partial<React.ComponentProps<typeof ItemCard>> = {}) {
  const defaultProps = {
    item: MOCK_ITEM,
    categories: MOCK_CATEGORIES,
    onPress: jest.fn(),
    ...props,
  };
  return render(
    <PaperProvider theme={paperTheme}>
      <ItemCard {...defaultProps} />
    </PaperProvider>,
  );
}

describe('ItemCard', () => {
  it('renders item name', () => {
    renderComponent();
    expect(screen.getByText('Nike Air Max')).toBeTruthy();
  });

  it('renders price when present', () => {
    renderComponent();
    expect(screen.getByText('150 \u20AC')).toBeTruthy();
  });

  it('does not render price when null', () => {
    renderComponent({ item: { ...MOCK_ITEM, estimated_price: null } });
    expect(screen.queryByText(/\u20AC/)).toBeNull();
  });

  it('renders category chip when category matches', () => {
    renderComponent();
    expect(screen.getByText('Mode')).toBeTruthy();
  });

  it('calls onPress with item id', () => {
    const onPress = jest.fn();
    renderComponent({ onPress });
    fireEvent.press(screen.getByTestId('item-card-item-1'));
    expect(onPress).toHaveBeenCalledWith('item-1');
  });

  it('renders priority indicator', () => {
    renderComponent();
    expect(screen.getByTestId('priority-indicator')).toBeTruthy();
  });

  it('renders age badge', () => {
    renderComponent();
    expect(screen.getByTestId('age-badge')).toBeTruthy();
  });
});
