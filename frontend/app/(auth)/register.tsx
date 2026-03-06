import { useState } from 'react';
import { TextInput, Button, HelperText } from 'react-native-paper';
import { router } from 'expo-router';
import { useTranslation } from 'react-i18next';

import { useAuthStore } from '@/src/stores/auth';
import { colors } from '@/src/theme';
import { ApiRequestError } from '@/src/api/client';
import { ROUTES } from '@/src/constants/routes';
import { AuthLayout, authStyles } from '@/src/components/auth';
import { validateEmail, validatePasswordMinLength } from '@/src/utils/validation';
import PasswordStrengthIndicator from '@/src/components/PasswordStrengthIndicator';

export default function RegisterScreen() {
  const { t } = useTranslation();
  const register = useAuthStore((s) => s.register);

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [emailError, setEmailError] = useState('');
  const [passwordError, setPasswordError] = useState('');
  const [confirmPasswordError, setConfirmPasswordError] = useState('');
  const [displayNameError, setDisplayNameError] = useState('');
  const [apiError, setApiError] = useState('');

  function handleValidateEmail(): boolean {
    const error = validateEmail(email, t);
    setEmailError(error ?? '');
    return !error;
  }

  function handleValidatePassword(): boolean {
    const error = validatePasswordMinLength(password, t);
    setPasswordError(error ?? '');
    return !error;
  }

  function handleValidateConfirmPassword(): boolean {
    if (confirmPassword !== password) {
      setConfirmPasswordError(t('auth.validation.passwordMismatch'));
      return false;
    }
    setConfirmPasswordError('');
    return true;
  }

  function handleValidateDisplayName(): boolean {
    if (displayName.length > 100) {
      setDisplayNameError(t('auth.register.displayNameMax'));
      return false;
    }
    setDisplayNameError('');
    return true;
  }

  async function handleRegister() {
    setApiError('');
    const emailValid = handleValidateEmail();
    const passwordValid = handleValidatePassword();
    const confirmValid = handleValidateConfirmPassword();
    const nameValid = handleValidateDisplayName();
    if (!emailValid || !passwordValid || !confirmValid || !nameValid) return;

    setIsSubmitting(true);
    try {
      await register(email.trim(), password, displayName.trim() || undefined);
      router.replace(ROUTES.HOME);
    } catch (error) {
      if (error instanceof ApiRequestError && error.status === 409) {
        setEmailError(t('auth.register.emailTaken'));
      } else if (error instanceof ApiRequestError && error.status === 400) {
        if (error.message.includes('common')) {
          setPasswordError(t('auth.validation.passwordCommon'));
        } else if (error.message.includes('breach')) {
          setPasswordError(t('auth.validation.passwordBreached'));
        } else {
          setApiError(error.message);
        }
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
      title={t('auth.register.title')}
      apiError={apiError}
      linkText={t('auth.register.hasAccount')}
      linkAction={t('auth.register.login')}
      onLinkPress={() => router.push(ROUTES.LOGIN)}
      linkTestID="goto-login"
    >
      <TextInput
        label={t('auth.register.emailLabel')}
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
        label={t('auth.register.passwordLabel')}
        value={password}
        onChangeText={(v) => {
          setPassword(v);
          if (passwordError) setPasswordError('');
        }}
        onBlur={handleValidatePassword}
        mode="outlined"
        secureTextEntry={!showPassword}
        error={!!passwordError}
        maxLength={128}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        right={
          <TextInput.Icon
            icon={showPassword ? 'eye-off-outline' : 'eye-outline'}
            onPress={() => setShowPassword(!showPassword)}
            accessibilityLabel={showPassword ? t('auth.register.hidePassword') : t('auth.register.showPassword')}
            testID="toggle-password"
          />
        }
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="password-input"
      />
      {passwordError ? (
        <HelperText type="error" visible testID="password-error">
          {passwordError}
        </HelperText>
      ) : (
        <PasswordStrengthIndicator password={password} />
      )}

      <TextInput
        label={t('auth.register.confirmPasswordLabel')}
        value={confirmPassword}
        onChangeText={(v) => {
          setConfirmPassword(v);
          if (confirmPasswordError) setConfirmPasswordError('');
        }}
        onBlur={handleValidateConfirmPassword}
        mode="outlined"
        secureTextEntry={!showPassword}
        error={!!confirmPasswordError}
        maxLength={128}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="confirm-password-input"
      />
      <HelperText type="error" visible={!!confirmPasswordError} testID="confirm-password-error">
        {confirmPasswordError}
      </HelperText>

      <TextInput
        label={t('auth.register.displayNameLabel')}
        value={displayName}
        onChangeText={(v) => {
          setDisplayName(v);
          if (displayNameError) setDisplayNameError('');
        }}
        onBlur={handleValidateDisplayName}
        mode="outlined"
        autoCapitalize="words"
        autoComplete="given-name"
        error={!!displayNameError}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="displayname-input"
      />
      <HelperText type="error" visible={!!displayNameError} testID="displayname-error">
        {displayNameError}
      </HelperText>

      <Button
        mode="contained"
        onPress={handleRegister}
        loading={isSubmitting}
        disabled={isSubmitting}
        style={authStyles.button}
        contentStyle={authStyles.buttonContent}
        labelStyle={authStyles.buttonLabel}
        testID="register-button"
      >
        {t('auth.register.submit')}
      </Button>
    </AuthLayout>
  );
}
