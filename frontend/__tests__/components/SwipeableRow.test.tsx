import { render, screen } from '@testing-library/react-native';
import { Text } from 'react-native';
import { SwipeableRow } from '@/src/components/items/SwipeableRow';

// gesture-handler is mocked in jest.setup.js (Swipeable → View)

describe('SwipeableRow', () => {
  it('renders children', () => {
    render(
      <SwipeableRow onSwipeLeft={jest.fn()} onSwipeRight={jest.fn()}>
        <Text>Child Content</Text>
      </SwipeableRow>,
    );
    expect(screen.getByText('Child Content')).toBeTruthy();
  });
});
