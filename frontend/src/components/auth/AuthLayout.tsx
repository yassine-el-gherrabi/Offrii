import type { ReactNode } from 'react';
import { View, KeyboardAvoidingView, ScrollView, Platform } from 'react-native';
import { Text } from 'react-native-paper';
import { SafeAreaView } from 'react-native-safe-area-context';
import { MaterialCommunityIcons } from '@expo/vector-icons';

import { colors } from '@/src/theme';
import { authStyles } from './styles';

interface AuthLayoutProps {
  title: string;
  apiError?: string;
  linkText: string;
  linkAction: string;
  onLinkPress: () => void;
  linkTestID: string;
  children: ReactNode;
}

export function AuthLayout({
  title,
  apiError,
  linkText,
  linkAction,
  onLinkPress,
  linkTestID,
  children,
}: AuthLayoutProps) {
  return (
    <SafeAreaView style={authStyles.safe}>
      <KeyboardAvoidingView
        style={authStyles.flex}
        behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      >
        <ScrollView
          contentContainerStyle={authStyles.scroll}
          keyboardShouldPersistTaps="handled"
        >
          <View style={authStyles.logoContainer}>
            <MaterialCommunityIcons
              name="gift-outline"
              size={64}
              color={colors.primary}
            />
          </View>

          <Text variant="headlineMedium" style={authStyles.title}>
            {title}
          </Text>

          {apiError ? (
            <View style={authStyles.errorBanner}>
              <Text style={authStyles.errorBannerText}>{apiError}</Text>
            </View>
          ) : null}

          {children}

          <View style={authStyles.linkRow}>
            <Text style={authStyles.linkText}>{linkText}</Text>
            <Text
              style={authStyles.link}
              onPress={onLinkPress}
              accessibilityRole="link"
              testID={linkTestID}
            >
              {linkAction}
            </Text>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}
