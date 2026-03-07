import { colors } from '@/src/theme';

export function daysAgo(isoDate: string): number {
  const then = new Date(isoDate);
  if (isNaN(then.getTime())) return 0;
  const diffMs = Date.now() - then.getTime();
  return Math.max(0, Math.floor(diffMs / (1000 * 60 * 60 * 24)));
}

export function ageColor(days: number): string {
  if (days < 7) return colors.ageFresh;
  if (days <= 30) return colors.ageModerate;
  return colors.ageOld;
}

export function formatRelativeDate(isoDate: string): string {
  return `${daysAgo(isoDate)}j`;
}
