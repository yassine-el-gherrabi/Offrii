import { render, screen, fireEvent, waitFor } from '@testing-library/react-native';
import { PaperProvider } from 'react-native-paper';
import { paperTheme } from '@/src/theme';
import { QuickCaptureInput } from '@/src/components/items/QuickCaptureInput';

function renderComponent(props: Partial<React.ComponentProps<typeof QuickCaptureInput>> = {}) {
  const defaultProps = {
    onSubmit: jest.fn().mockResolvedValue(undefined),
    isSubmitting: false,
    ...props,
  };
  return render(
    <PaperProvider theme={paperTheme}>
      <QuickCaptureInput {...defaultProps} />
    </PaperProvider>,
  );
}

beforeEach(() => {
  jest.clearAllMocks();
});

describe('QuickCaptureInput', () => {
  it('renders the input', () => {
    renderComponent();
    expect(screen.getByTestId('quick-capture-input')).toBeTruthy();
  });

  it('calls onSubmit with trimmed text on submit', async () => {
    const onSubmit = jest.fn().mockResolvedValue(undefined);
    renderComponent({ onSubmit });

    fireEvent.changeText(screen.getByTestId('quick-capture-input'), '  Nike Air Max  ');
    fireEvent(screen.getByTestId('quick-capture-input'), 'submitEditing');

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith('Nike Air Max');
    });
  });

  it('does not submit empty text', async () => {
    const onSubmit = jest.fn();
    renderComponent({ onSubmit });

    fireEvent.changeText(screen.getByTestId('quick-capture-input'), '   ');
    fireEvent(screen.getByTestId('quick-capture-input'), 'submitEditing');

    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('does not submit while isSubmitting', async () => {
    const onSubmit = jest.fn();
    renderComponent({ onSubmit, isSubmitting: true });

    fireEvent.changeText(screen.getByTestId('quick-capture-input'), 'Test');
    fireEvent(screen.getByTestId('quick-capture-input'), 'submitEditing');

    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('clears input after successful submit', async () => {
    const onSubmit = jest.fn().mockResolvedValue(undefined);
    renderComponent({ onSubmit });

    fireEvent.changeText(screen.getByTestId('quick-capture-input'), 'Test Item');
    fireEvent(screen.getByTestId('quick-capture-input'), 'submitEditing');

    await waitFor(() => {
      expect(screen.getByTestId('quick-capture-input').props.value).toBe('');
    });
  });
});
