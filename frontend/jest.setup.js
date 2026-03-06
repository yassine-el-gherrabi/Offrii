// Mock react-native-safe-area-context globally for tests
jest.mock('react-native-safe-area-context', () => {
  const React = require('react');

  const insets = { top: 0, right: 0, bottom: 0, left: 0 };
  const frame = { x: 0, y: 0, width: 320, height: 640 };

  const SafeAreaInsetsContext = React.createContext(insets);
  const SafeAreaFrameContext = React.createContext(frame);

  return {
    SafeAreaProvider: ({ children }) => React.createElement(React.Fragment, null, children),
    SafeAreaView: ({ children, ...props }) =>
      React.createElement(require('react-native').View, props, children),
    useSafeAreaInsets: () => insets,
    useSafeAreaFrame: () => frame,
    SafeAreaInsetsContext,
    SafeAreaFrameContext,
    initialWindowMetrics: { frame, insets },
  };
});
