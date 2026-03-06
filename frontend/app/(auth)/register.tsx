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

import { useAuthStore } from '@/src/stores/auth';
import { colors, spacing, borderRadius } from '@/src/theme';
import { ApiRequestError } from '@/src/api/client';

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export default function RegisterScreen() {
  const register = useAuthStore((s) => s.register);

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [emailError, setEmailError] = useState('');
  const [passwordError, setPasswordError] = useState('');
  const [displayNameError, setDisplayNameError] = useState('');
  const [apiError, setApiError] = useState('');

  function validateEmail(): boolean {
    if (!email.trim()) {
      setEmailError('L\'email est requis');
      return false;
    }
    if (!EMAIL_REGEX.test(email.trim())) {
      setEmailError('Format d\'email invalide');
      return false;
    }
    setEmailError('');
    return true;
  }

  function validatePassword(): boolean {
    if (!password) {
      setPasswordError('Le mot de passe est requis');
      return false;
    }
    if (password.length < 8) {
      setPasswordError('8 caractères minimum');
      return false;
    }
    setPasswordError('');
    return true;
  }

  function validateDisplayName(): boolean {
    if (displayName.length > 100) {
      setDisplayNameError('100 caractères maximum');
      return false;
    }
    setDisplayNameError('');
    return true;
  }

  async function handleRegister() {
    setApiError('');
    const emailValid = validateEmail();
    const passwordValid = validatePassword();
    const nameValid = validateDisplayName();
    if (!emailValid || !passwordValid || !nameValid) return;

    setIsSubmitting(true);
    try {
      await register(email.trim(), password, displayName.trim() || undefined);
      router.replace('/(tabs)/capture');
    } catch (error) {
      if (error instanceof ApiRequestError && error.status === 409) {
        setEmailError('Cet email est déjà utilisé');
      } else if (error instanceof ApiRequestError) {
        setApiError(error.message);
      } else {
        setApiError('Une erreur inattendue est survenue');
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
            {"Rejoins l'aventure"}{'\n'}{"Offrii ! 🎉"}
          </Text>

          {/* API error banner */}
          {apiError ? (
            <View style={styles.errorBanner}>
              <Text style={styles.errorBannerText}>{apiError}</Text>
            </View>
          ) : null}

          {/* Email */}
          <TextInput
            label="Email"
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
            left={<TextInput.Icon icon="email-outline" />}
            style={styles.input}
            outlineStyle={styles.inputOutline}
            testID="email-input"
          />
          <HelperText type="error" visible={!!emailError} testID="email-error">
            {emailError}
          </HelperText>

          {/* Password */}
          <TextInput
            label="Mot de passe"
            value={password}
            onChangeText={(v) => {
              setPassword(v);
              if (passwordError) setPasswordError('');
            }}
            onBlur={validatePassword}
            mode="outlined"
            secureTextEntry={!showPassword}
            error={!!passwordError}
            left={<TextInput.Icon icon="lock-outline" />}
            right={
              <TextInput.Icon
                icon={showPassword ? 'eye-off-outline' : 'eye-outline'}
                onPress={() => setShowPassword(!showPassword)}
                testID="toggle-password"
              />
            }
            style={styles.input}
            outlineStyle={styles.inputOutline}
            testID="password-input"
          />
          <HelperText
            type={passwordError ? 'error' : 'info'}
            visible={!!passwordError || password.length === 0}
            testID="password-helper"
          >
            {passwordError || '8 caractères minimum'}
          </HelperText>

          {/* Display name */}
          <TextInput
            label="Prénom (optionnel)"
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
            left={<TextInput.Icon icon="emoticon-happy-outline" />}
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
            {"S'inscrire"}
          </Button>

          {/* Link to login */}
          <View style={styles.linkRow}>
            <Text style={styles.linkText}>Déjà un compte ? </Text>
            <Text
              style={styles.link}
              onPress={() => router.push('/(auth)/login')}
              testID="goto-login"
            >
              Se connecter
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
    backgroundColor: colors.surface,
  },
  inputOutline: {
    borderRadius: borderRadius.md,
  },
  button: {
    marginTop: spacing.sm,
    borderRadius: borderRadius.lg,
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
    color: colors.secondary,
    fontWeight: '600',
  },
});
