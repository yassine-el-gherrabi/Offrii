import { daysAgo, ageColor, formatRelativeDate } from '@/src/utils/dates';
import { colors } from '@/src/theme';

describe('daysAgo', () => {
  it('returns 0 for today', () => {
    const today = new Date().toISOString();
    expect(daysAgo(today)).toBe(0);
  });

  it('returns 1 for yesterday', () => {
    const yesterday = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
    expect(daysAgo(yesterday)).toBe(1);
  });

  it('returns 0 for future dates', () => {
    const tomorrow = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();
    expect(daysAgo(tomorrow)).toBe(0);
  });

  it('returns 0 for invalid date strings', () => {
    expect(daysAgo('invalid')).toBe(0);
    expect(daysAgo('')).toBe(0);
    expect(daysAgo('null')).toBe(0);
  });

  it('returns correct count for 30 days ago', () => {
    const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString();
    expect(daysAgo(thirtyDaysAgo)).toBe(30);
  });
});

describe('ageColor', () => {
  it('returns ageFresh for < 7 days', () => {
    expect(ageColor(0)).toBe(colors.ageFresh);
    expect(ageColor(6)).toBe(colors.ageFresh);
  });

  it('returns ageModerate for 7-30 days', () => {
    expect(ageColor(7)).toBe(colors.ageModerate);
    expect(ageColor(30)).toBe(colors.ageModerate);
  });

  it('returns ageOld for > 30 days', () => {
    expect(ageColor(31)).toBe(colors.ageOld);
    expect(ageColor(100)).toBe(colors.ageOld);
  });
});

describe('formatRelativeDate', () => {
  it('formats today as 0j by default', () => {
    const today = new Date().toISOString();
    expect(formatRelativeDate(today)).toBe('0j');
  });

  it('formats with j suffix by default', () => {
    const fiveDaysAgo = new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString();
    expect(formatRelativeDate(fiveDaysAgo)).toBe('5j');
  });

  it('accepts a custom suffix', () => {
    const today = new Date().toISOString();
    expect(formatRelativeDate(today, 'd')).toBe('0d');
  });
});
