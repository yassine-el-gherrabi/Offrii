import { render, screen } from '@testing-library/react-native';
import { PriorityIndicator } from '@/src/components/items/PriorityIndicator';
import { colors } from '@/src/theme';

describe('PriorityIndicator', () => {
  it('renders a dot', () => {
    render(<PriorityIndicator priority={2} />);
    expect(screen.getByTestId('priority-indicator')).toBeTruthy();
  });

  it('uses error color for high priority (3)', () => {
    render(<PriorityIndicator priority={3} />);
    const dot = screen.getByTestId('priority-indicator');
    expect(dot.props.style).toEqual(
      expect.arrayContaining([expect.objectContaining({ backgroundColor: colors.error })]),
    );
  });

  it('uses ageModerate color for medium priority (2)', () => {
    render(<PriorityIndicator priority={2} />);
    const dot = screen.getByTestId('priority-indicator');
    expect(dot.props.style).toEqual(
      expect.arrayContaining([expect.objectContaining({ backgroundColor: colors.ageModerate })]),
    );
  });

  it('uses textSecondary color for low priority (1)', () => {
    render(<PriorityIndicator priority={1} />);
    const dot = screen.getByTestId('priority-indicator');
    expect(dot.props.style).toEqual(
      expect.arrayContaining([expect.objectContaining({ backgroundColor: colors.textSecondary })]),
    );
  });
});
