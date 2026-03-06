export const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export function validateEmail(
  email: string,
  t: (key: string) => string,
): string | null {
  if (!email.trim()) return t('auth.validation.emailRequired');
  if (!EMAIL_REGEX.test(email.trim())) return t('auth.validation.emailInvalid');
  return null;
}

export function validatePasswordRequired(
  password: string,
  t: (key: string) => string,
): string | null {
  if (!password) return t('auth.validation.passwordRequired');
  return null;
}

export function validatePasswordMinLength(
  password: string,
  t: (key: string) => string,
): string | null {
  if (!password) return t('auth.validation.passwordRequired');
  if (password.length < 8) return t('auth.validation.passwordMinLength');
  return null;
}
