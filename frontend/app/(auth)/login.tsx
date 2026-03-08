import { useState } from 'react';
import { Text } from 'react-native';
import { TextInput, Button, HelperText } from 'react-native-paper';
import { router } from 'expo-router';
import { useTranslation } from 'react-i18next';

import { useAuthStore } from '@/src/stores/auth';
import { colors, spacing } from '@/src/theme';
import { ApiRequestError } from '@/src/api/client';
import { ROUTES } from '@/src/constants/routes';
import { AuthLayout, authStyles } from '@/src/components/auth';
import { validateEmail, validatePasswordRequired } from '@/src/utils/validation';

export default function LoginScreen() {
  const { t } = useTranslation();
  const login = useAuthStore((s) => s.login);

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [emailError, setEmailError] = useState('');
  const [passwordError, setPasswordError] = useState('');
  const [apiError, setApiError] = useState('');

  function handleValidateEmail(): boolean {
    const error = validateEmail(email, t);
    setEmailError(error ?? '');
    return !error;
  }

  function handleValidatePassword(): boolean {
    const error = validatePasswordRequired(password, t);
    setPasswordError(error ?? '');
    return !error;
  }

  async function handleLogin() {
    setApiError('');
    const emailValid = handleValidateEmail();
    const passwordValid = handleValidatePassword();
    if (!emailValid || !passwordValid) return;

    setIsSubmitting(true);
    try {
      await login(email.trim(), password);
      router.replace(ROUTES.HOME);
    } catch (error) {
      if (error instanceof ApiRequestError && error.status === 401) {
        setApiError(t('auth.login.invalidCredentials'));
      } else if (error instanceof ApiRequestError && error.status === 0) {
        setApiError(t('auth.errors.networkError'));
      } else if (error instanceof ApiRequestError) {
        setApiError(error.message);
      } else {
        setApiError(t('auth.errors.unexpected'));
      }
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <AuthLayout
      title={t('auth.login.title')}
      apiError={apiError}
      linkText={t('auth.login.noAccount')}
      linkAction={t('auth.login.createAccount')}
      onLinkPress={() => router.push(ROUTES.REGISTER)}
      linkTestID="goto-register"
    >
      <TextInput
        label={t('auth.login.emailLabel')}
        value={email}
        onChangeText={(v) => {
          setEmail(v);
          if (emailError) setEmailError('');
        }}
        onBlur={handleValidateEmail}
        mode="outlined"
        keyboardType="email-address"
        autoCapitalize="none"
        autoComplete="email"
        error={!!emailError}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="email-input"
      />
      <HelperText type="error" visible={!!emailError} testID="email-error">
        {emailError}
      </HelperText>

      <TextInput
        label={t('auth.login.passwordLabel')}
        value={password}
        onChangeText={(v) => {
          setPassword(v);
          if (passwordError) setPasswordError('');
        }}
        onBlur={handleValidatePassword}
        mode="outlined"
        secureTextEntry={!showPassword}
        error={!!passwordError}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        right={
          <TextInput.Icon
            icon={showPassword ? 'eye-off-outline' : 'eye-outline'}
            onPress={() => setShowPassword(!showPassword)}
            accessibilityLabel={showPassword ? t('auth.login.hidePassword') : t('auth.login.showPassword')}
            testID="toggle-password"
          />
        }
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="password-input"
      />
      <HelperText type="error" visible={!!passwordError} testID="password-error">
        {passwordError}
      </HelperText>

      <Text
        onPress={() => router.push(ROUTES.FORGOT_PASSWORD)}
        style={{
          color: colors.primary,
          textAlign: 'right',
          marginBottom: spacing.sm,
          fontWeight: '600',
        }}
        testID="forgot-password-link"
      >
        {t('auth.forgotPassword.link')}
      </Text>

      <Button
        mode="contained"
        onPress={handleLogin}
        loading={isSubmitting}
        disabled={isSubmitting}
        style={authStyles.button}
        contentStyle={authStyles.buttonContent}
        labelStyle={authStyles.buttonLabel}
        testID="login-button"
      >
        {t('auth.login.submit')}
      </Button>
    </AuthLayout>
  );
}
