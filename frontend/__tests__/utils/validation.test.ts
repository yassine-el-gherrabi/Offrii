import { EMAIL_REGEX, validateEmail, validatePasswordRequired, validatePasswordMinLength } from '@/src/utils/validation';

const t = (key: string) => key;

describe('EMAIL_REGEX', () => {
  it('matches valid emails', () => {
    expect(EMAIL_REGEX.test('user@example.com')).toBe(true);
    expect(EMAIL_REGEX.test('a@b.co')).toBe(true);
    expect(EMAIL_REGEX.test('user+tag@domain.org')).toBe(true);
  });

  it('rejects invalid emails', () => {
    expect(EMAIL_REGEX.test('')).toBe(false);
    expect(EMAIL_REGEX.test('notanemail')).toBe(false);
    expect(EMAIL_REGEX.test('@domain.com')).toBe(false);
    expect(EMAIL_REGEX.test('user@')).toBe(false);
    expect(EMAIL_REGEX.test('user @domain.com')).toBe(false);
  });
});

describe('validateEmail', () => {
  it('returns error key for empty email', () => {
    expect(validateEmail('', t)).toBe('auth.validation.emailRequired');
    expect(validateEmail('   ', t)).toBe('auth.validation.emailRequired');
  });

  it('returns error key for invalid format', () => {
    expect(validateEmail('notanemail', t)).toBe('auth.validation.emailInvalid');
  });

  it('returns null for valid email', () => {
    expect(validateEmail('user@example.com', t)).toBeNull();
  });

  it('trims whitespace before validation', () => {
    expect(validateEmail(' user@example.com ', t)).toBeNull();
  });
});

describe('validatePasswordRequired', () => {
  it('returns error key for empty password', () => {
    expect(validatePasswordRequired('', t)).toBe('auth.validation.passwordRequired');
  });

  it('returns null for any non-empty password', () => {
    expect(validatePasswordRequired('a', t)).toBeNull();
  });
});

describe('validatePasswordMinLength', () => {
  it('returns error key for empty password', () => {
    expect(validatePasswordMinLength('', t)).toBe('auth.validation.passwordRequired');
  });

  it('returns error key for short password', () => {
    expect(validatePasswordMinLength('short', t)).toBe('auth.validation.passwordMinLength');
  });

  it('returns null for 8+ char password', () => {
    expect(validatePasswordMinLength('12345678', t)).toBeNull();
  });
});
