import { render, screen } from '@testing-library/react-native';
import { Text } from 'react-native';

describe('Test setup', () => {
  it('renders correctly', () => {
    render(<Text>Hello</Text>);
    expect(screen.getByText('Hello')).toBeTruthy();
  });
});
