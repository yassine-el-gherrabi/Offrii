import { useState } from 'react';
import {
  View,
  StyleSheet,
  KeyboardAvoidingView,
  ScrollView,
  Platform,
} from 'react-native';
import { TextInput, Button, Text, HelperText } from 'react-native-paper';
import { SafeAreaView } from 'react-native-safe-area-context';
import { MaterialCommunityIcons } from '@expo/vector-icons';
import { router } from 'expo-router';
import { useTranslation } from 'react-i18next';

import { useAuthStore } from '@/src/stores/auth';
import { colors, spacing, borderRadius } from '@/src/theme';
import { ApiRequestError } from '@/src/api/client';
import { ROUTES } from '@/src/constants/routes';
import PasswordStrengthIndicator from '@/src/components/PasswordStrengthIndicator';

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

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

  function validateEmail(): boolean {
    if (!email.trim()) {
      setEmailError(t('auth.validation.emailRequired'));
      return false;
    }
    if (!EMAIL_REGEX.test(email.trim())) {
      setEmailError(t('auth.validation.emailInvalid'));
      return false;
    }
    setEmailError('');
    return true;
  }

  function validatePassword(): boolean {
    if (!password) {
      setPasswordError(t('auth.validation.passwordRequired'));
      return false;
    }
    if (password.length < 8) {
      setPasswordError(t('auth.validation.passwordMinLength'));
      return false;
    }
    setPasswordError('');
    return true;
  }

  function validateConfirmPassword(): boolean {
    if (confirmPassword !== password) {
      setConfirmPasswordError(t('auth.validation.passwordMismatch'));
      return false;
    }
    setConfirmPasswordError('');
    return true;
  }

  function validateDisplayName(): boolean {
    if (displayName.length > 100) {
      setDisplayNameError(t('auth.register.displayNameMax'));
      return false;
    }
    setDisplayNameError('');
    return true;
  }

  async function handleRegister() {
    setApiError('');
    const emailValid = validateEmail();
    const passwordValid = validatePassword();
    const confirmValid = validateConfirmPassword();
    const nameValid = validateDisplayName();
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
    <SafeAreaView style={styles.safe}>
      <KeyboardAvoidingView
        style={styles.flex}
        behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      >
        <ScrollView
          contentContainerStyle={styles.scroll}
          keyboardShouldPersistTaps="handled"
        >
          {/* Logo */}
          <View style={styles.logoContainer}>
            <MaterialCommunityIcons
              name="gift-outline"
              size={64}
              color={colors.primary}
            />
          </View>

          {/* Title */}
          <Text variant="headlineMedium" style={styles.title}>
            {t('auth.register.title')}
          </Text>

          {/* API error banner */}
          {apiError ? (
            <View style={styles.errorBanner}>
              <Text style={styles.errorBannerText}>{apiError}</Text>
            </View>
          ) : null}

          {/* Email */}
          <TextInput
            label={t('auth.register.emailLabel')}
            value={email}
            onChangeText={(v) => {
              setEmail(v);
              if (emailError) setEmailError('');
            }}
            onBlur={validateEmail}
            mode="outlined"
            keyboardType="email-address"
            autoCapitalize="none"
            autoComplete="email"
            error={!!emailError}
            outlineColor={colors.inputBorder}
            activeOutlineColor={colors.primary}
            style={styles.input}
            outlineStyle={styles.inputOutline}
            testID="email-input"
          />
          <HelperText type="error" visible={!!emailError} testID="email-error">
            {emailError}
          </HelperText>

          {/* Password */}
          <TextInput
            label={t('auth.register.passwordLabel')}
            value={password}
            onChangeText={(v) => {
              setPassword(v);
              if (passwordError) setPasswordError('');
            }}
            onBlur={validatePassword}
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
            style={styles.input}
            outlineStyle={styles.inputOutline}
            testID="password-input"
          />
          {passwordError ? (
            <HelperText type="error" visible testID="password-error">
              {passwordError}
            </HelperText>
          ) : (
            <PasswordStrengthIndicator password={password} />
          )}

          {/* Confirm password */}
          <TextInput
            label={t('auth.register.confirmPasswordLabel')}
            value={confirmPassword}
            onChangeText={(v) => {
              setConfirmPassword(v);
              if (confirmPasswordError) setConfirmPasswordError('');
            }}
            onBlur={validateConfirmPassword}
            mode="outlined"
            secureTextEntry={!showPassword}
            error={!!confirmPasswordError}
            maxLength={128}
            outlineColor={colors.inputBorder}
            activeOutlineColor={colors.primary}
            style={styles.input}
            outlineStyle={styles.inputOutline}
            testID="confirm-password-input"
          />
          <HelperText type="error" visible={!!confirmPasswordError} testID="confirm-password-error">
            {confirmPasswordError}
          </HelperText>

          {/* Display name */}
          <TextInput
            label={t('auth.register.displayNameLabel')}
            value={displayName}
            onChangeText={(v) => {
              setDisplayName(v);
              if (displayNameError) setDisplayNameError('');
            }}
            onBlur={validateDisplayName}
            mode="outlined"
            autoCapitalize="words"
            autoComplete="given-name"
            error={!!displayNameError}
            outlineColor={colors.inputBorder}
            activeOutlineColor={colors.primary}
            style={styles.input}
            outlineStyle={styles.inputOutline}
            testID="displayname-input"
          />
          <HelperText type="error" visible={!!displayNameError} testID="displayname-error">
            {displayNameError}
          </HelperText>

          {/* Submit */}
          <Button
            mode="contained"
            onPress={handleRegister}
            loading={isSubmitting}
            disabled={isSubmitting}
            style={styles.button}
            contentStyle={styles.buttonContent}
            labelStyle={styles.buttonLabel}
            testID="register-button"
          >
            {t('auth.register.submit')}
          </Button>

          {/* Link to login */}
          <View style={styles.linkRow}>
            <Text style={styles.linkText}>{t('auth.register.hasAccount')}</Text>
            <Text
              style={styles.link}
              onPress={() => router.push('/(auth)/login')}
              testID="goto-login"
            >
              {t('auth.register.login')}
            </Text>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  flex: {
    flex: 1,
  },
  scroll: {
    flexGrow: 1,
    justifyContent: 'center',
    paddingHorizontal: spacing.lg,
    paddingVertical: spacing.xl,
  },
  logoContainer: {
    alignItems: 'center',
    marginBottom: spacing.md,
  },
  title: {
    textAlign: 'center',
    fontWeight: '700',
    color: colors.text,
    marginBottom: spacing.lg,
  },
  errorBanner: {
    backgroundColor: '#FEE2E2',
    borderRadius: borderRadius.sm,
    padding: spacing.md,
    marginBottom: spacing.md,
  },
  errorBannerText: {
    color: colors.error,
    textAlign: 'center',
    fontSize: 14,
  },
  input: {
    backgroundColor: colors.inputBackground,
  },
  inputOutline: {
    borderRadius: borderRadius.sm,
  },
  button: {
    marginTop: spacing.sm,
    borderRadius: borderRadius.sm,
    backgroundColor: colors.primary,
  },
  buttonContent: {
    paddingVertical: spacing.sm,
  },
  buttonLabel: {
    fontSize: 16,
    fontWeight: '600',
  },
  linkRow: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginTop: spacing.lg,
  },
  linkText: {
    color: colors.textSecondary,
  },
  link: {
    color: colors.primary,
    fontWeight: '600',
  },
});
