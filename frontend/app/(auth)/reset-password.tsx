import { useState, useEffect, useCallback } from 'react';
import { TextInput, Button, HelperText } from 'react-native-paper';
import { router, useLocalSearchParams } from 'expo-router';
import { useTranslation } from 'react-i18next';

import { colors } from '@/src/theme';
import { ApiRequestError } from '@/src/api/client';
import { ROUTES } from '@/src/constants/routes';
import { AuthLayout, authStyles } from '@/src/components/auth';
import { resetPassword, forgotPassword } from '@/src/api/auth';

export default function ResetPasswordScreen() {
  const { t } = useTranslation();
  const { email } = useLocalSearchParams<{ email: string }>();

  const [code, setCode] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [codeError, setCodeError] = useState('');
  const [passwordError, setPasswordError] = useState('');
  const [confirmError, setConfirmError] = useState('');
  const [apiError, setApiError] = useState('');
  const [successMessage, setSuccessMessage] = useState('');

  // Resend cooldown
  const [resendCooldown, setResendCooldown] = useState(0);
  const [resending, setResending] = useState(false);

  useEffect(() => {
    if (resendCooldown <= 0) return;
    const timer = setTimeout(() => setResendCooldown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [resendCooldown]);

  function validate(): boolean {
    let valid = true;

    if (!/^\d{6}$/.test(code)) {
      setCodeError(t('auth.resetPassword.invalidCode'));
      valid = false;
    } else {
      setCodeError('');
    }

    if (newPassword.length < 8) {
      setPasswordError(t('auth.validation.passwordMinLength'));
      valid = false;
    } else {
      setPasswordError('');
    }

    if (newPassword !== confirmPassword) {
      setConfirmError(t('auth.resetPassword.passwordMismatch'));
      valid = false;
    } else {
      setConfirmError('');
    }

    return valid;
  }

  async function handleSubmit() {
    setApiError('');
    setSuccessMessage('');
    if (!validate()) return;

    setIsSubmitting(true);
    try {
      await resetPassword(email!, code, newPassword);
      setSuccessMessage(t('auth.resetPassword.success'));
      setTimeout(() => {
        router.replace(ROUTES.LOGIN);
      }, 1500);
    } catch (error) {
      if (error instanceof ApiRequestError && error.status === 400) {
        setApiError(t('auth.resetPassword.invalidCode'));
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

  const handleResend = useCallback(async () => {
    if (!email || resendCooldown > 0) return;
    setResending(true);
    try {
      await forgotPassword(email);
      setResendCooldown(60);
    } catch {
      // Silently fail — the user can retry
    } finally {
      setResending(false);
    }
  }, [email, resendCooldown]);

  return (
    <AuthLayout
      title={t('auth.resetPassword.title')}
      apiError={apiError || successMessage}
      linkText=""
      linkAction={t('auth.login.submit')}
      onLinkPress={() => router.replace(ROUTES.LOGIN)}
      linkTestID="goto-login"
    >
      <TextInput
        label={t('auth.resetPassword.codeLabel')}
        value={code}
        onChangeText={(v) => {
          setCode(v);
          if (codeError) setCodeError('');
        }}
        mode="outlined"
        keyboardType="number-pad"
        maxLength={6}
        error={!!codeError}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="code-input"
      />
      <HelperText type="error" visible={!!codeError} testID="code-error">
        {codeError}
      </HelperText>

      <TextInput
        label={t('auth.resetPassword.newPasswordLabel')}
        value={newPassword}
        onChangeText={(v) => {
          setNewPassword(v);
          if (passwordError) setPasswordError('');
        }}
        mode="outlined"
        secureTextEntry={!showPassword}
        error={!!passwordError}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        right={
          <TextInput.Icon
            icon={showPassword ? 'eye-off-outline' : 'eye-outline'}
            onPress={() => setShowPassword(!showPassword)}
            testID="toggle-password"
          />
        }
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="new-password-input"
      />
      <HelperText type="error" visible={!!passwordError} testID="password-error">
        {passwordError}
      </HelperText>

      <TextInput
        label={t('auth.resetPassword.confirmPasswordLabel')}
        value={confirmPassword}
        onChangeText={(v) => {
          setConfirmPassword(v);
          if (confirmError) setConfirmError('');
        }}
        mode="outlined"
        secureTextEntry={!showPassword}
        error={!!confirmError}
        outlineColor={colors.inputBorder}
        activeOutlineColor={colors.primary}
        style={authStyles.input}
        outlineStyle={authStyles.inputOutline}
        testID="confirm-password-input"
      />
      <HelperText type="error" visible={!!confirmError} testID="confirm-error">
        {confirmError}
      </HelperText>

      <Button
        mode="contained"
        onPress={handleSubmit}
        loading={isSubmitting}
        disabled={isSubmitting}
        style={authStyles.button}
        contentStyle={authStyles.buttonContent}
        labelStyle={authStyles.buttonLabel}
        testID="reset-button"
      >
        {t('auth.resetPassword.submit')}
      </Button>

      <Button
        mode="text"
        onPress={handleResend}
        disabled={resendCooldown > 0 || resending}
        loading={resending}
        style={{ marginTop: 12 }}
        testID="resend-button"
      >
        {resendCooldown > 0
          ? t('auth.resetPassword.resendIn', { seconds: resendCooldown })
          : t('auth.resetPassword.resendCode')}
      </Button>
    </AuthLayout>
  );
}
