import { useState } from 'react';
import { TextInput, Button, HelperText } from 'react-native-paper';
import { router } from 'expo-router';
import { useTranslation } from 'react-i18next';

import { colors } from '@/src/theme';
import { ApiRequestError } from '@/src/api/client';
import { ROUTES } from '@/src/constants/routes';
import { AuthLayout, authStyles } from '@/src/components/auth';
import { validateEmail } from '@/src/utils/validation';
import { forgotPassword } from '@/src/api/auth';

export default function ForgotPasswordScreen() {
  const { t } = useTranslation();

  const [email, setEmail] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [emailError, setEmailError] = useState('');
  const [apiError, setApiError] = useState('');

  function handleValidateEmail(): boolean {
    const error = validateEmail(email, t);
    setEmailError(error ?? '');
    return !error;
  }

  async function handleSubmit() {
    setApiError('');
    if (!handleValidateEmail()) return;

    setIsSubmitting(true);
    try {
      await forgotPassword(email.trim());
      router.push({
        pathname: ROUTES.RESET_PASSWORD,
        params: { email: email.trim() },
      });
    } catch (error) {
      if (error instanceof ApiRequestError && error.status === 0) {
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
      title={t('auth.forgotPassword.title')}
      apiError={apiError}
      linkText=""
      linkAction={t('auth.login.submit')}
      onLinkPress={() => router.back()}
      linkTestID="goto-login"
    >
      <HelperText type="info" visible style={{ marginBottom: 8 }}>
        {t('auth.forgotPassword.description')}
      </HelperText>

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

      <Button
        mode="contained"
        onPress={handleSubmit}
        loading={isSubmitting}
        disabled={isSubmitting}
        style={authStyles.button}
        contentStyle={authStyles.buttonContent}
        labelStyle={authStyles.buttonLabel}
        testID="send-code-button"
      >
        {t('auth.forgotPassword.sendCode')}
      </Button>
    </AuthLayout>
  );
}
