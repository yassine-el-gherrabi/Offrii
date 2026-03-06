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

// Mock react-i18next — resolve keys from French translations, falling back to the key itself
jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key) => {
      // Load the French translations and resolve nested keys
      const fr = require('./src/i18n/locales/fr.json');
      const parts = key.split('.');
      let value = fr;
      for (const part of parts) {
        value = value?.[part];
      }
      return value ?? key;
    },
    i18n: { changeLanguage: jest.fn(), language: 'fr' },
  }),
  initReactI18next: { type: '3rdParty', init: jest.fn() },
}));

// Mock expo-localization
jest.mock('expo-localization', () => ({
  getLocales: () => [{ languageCode: 'fr' }],
  getCalendars: () => [{}],
}));
