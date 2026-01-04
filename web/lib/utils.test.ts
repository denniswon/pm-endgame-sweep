import { describe, it, expect } from 'vitest';
import {
  cn,
  formatPercent,
  formatDuration,
  formatScore,
  getRiskColor,
  getSeverityColor,
} from './utils';

describe('cn (className utility)', () => {
  it('merges class names correctly', () => {
    expect(cn('text-red-500', 'bg-blue-500')).toBe('text-red-500 bg-blue-500');
  });

  it('handles conditional classes', () => {
    expect(cn('base', false && 'conditional', 'always')).toBe('base always');
  });

  it('handles Tailwind conflicts with tailwind-merge', () => {
    expect(cn('px-2', 'px-4')).toBe('px-4'); // Later class wins
  });

  it('handles undefined and null', () => {
    expect(cn('base', undefined, null, 'end')).toBe('base end');
  });
});

describe('formatPercent', () => {
  it('formats decimal to percentage with 2 decimals', () => {
    expect(formatPercent(0.1234)).toBe('12.34%');
  });

  it('handles zero correctly', () => {
    expect(formatPercent(0)).toBe('0.00%');
  });

  it('handles 1.0 correctly', () => {
    expect(formatPercent(1.0)).toBe('100.00%');
  });

  it('handles small decimals', () => {
    expect(formatPercent(0.0001)).toBe('0.01%');
  });

  it('handles large numbers', () => {
    expect(formatPercent(1.5)).toBe('150.00%');
  });

  it('handles negative numbers', () => {
    expect(formatPercent(-0.25)).toBe('-25.00%');
  });
});

describe('formatDuration', () => {
  it('formats days and hours', () => {
    const seconds = 86400 + 3600; // 1 day + 1 hour
    expect(formatDuration(seconds)).toBe('1d 1h');
  });

  it('formats multiple days', () => {
    const seconds = 86400 * 5 + 3600 * 3; // 5 days + 3 hours
    expect(formatDuration(seconds)).toBe('5d 3h');
  });

  it('formats hours and minutes', () => {
    const seconds = 3600 + 120; // 1 hour + 2 minutes
    expect(formatDuration(seconds)).toBe('1h 2m');
  });

  it('formats only minutes', () => {
    expect(formatDuration(120)).toBe('2m');
  });

  it('formats zero seconds', () => {
    expect(formatDuration(0)).toBe('0m');
  });

  it('handles large durations', () => {
    const seconds = 86400 * 365; // 1 year
    expect(formatDuration(seconds)).toBe('365d 0h');
  });
});

describe('formatScore', () => {
  it('formats score to 3 decimals', () => {
    expect(formatScore(0.12345)).toBe('0.123');
  });

  it('handles zero', () => {
    expect(formatScore(0)).toBe('0.000');
  });

  it('handles 1.0', () => {
    expect(formatScore(1.0)).toBe('1.000');
  });

  it('rounds correctly', () => {
    expect(formatScore(0.12345)).toBe('0.123');
    expect(formatScore(0.12355)).toBe('0.124');
  });

  it('handles negative scores', () => {
    expect(formatScore(-0.5)).toBe('-0.500');
  });
});

describe('getRiskColor', () => {
  it('returns red for high risk (>= 0.7)', () => {
    expect(getRiskColor(0.7)).toBe('text-red-600 bg-red-50');
    expect(getRiskColor(0.8)).toBe('text-red-600 bg-red-50');
    expect(getRiskColor(1.0)).toBe('text-red-600 bg-red-50');
  });

  it('returns yellow for medium risk (0.4-0.7)', () => {
    expect(getRiskColor(0.4)).toBe('text-yellow-600 bg-yellow-50');
    expect(getRiskColor(0.5)).toBe('text-yellow-600 bg-yellow-50');
    expect(getRiskColor(0.69)).toBe('text-yellow-600 bg-yellow-50');
  });

  it('returns green for low risk (< 0.4)', () => {
    expect(getRiskColor(0)).toBe('text-green-600 bg-green-50');
    expect(getRiskColor(0.2)).toBe('text-green-600 bg-green-50');
    expect(getRiskColor(0.39)).toBe('text-green-600 bg-green-50');
  });
});

describe('getSeverityColor', () => {
  it('returns red for high severity', () => {
    expect(getSeverityColor('high')).toBe('text-red-600 bg-red-50');
    expect(getSeverityColor('HIGH')).toBe('text-red-600 bg-red-50');
    expect(getSeverityColor('High')).toBe('text-red-600 bg-red-50');
  });

  it('returns yellow for medium severity', () => {
    expect(getSeverityColor('medium')).toBe('text-yellow-600 bg-yellow-50');
    expect(getSeverityColor('MEDIUM')).toBe('text-yellow-600 bg-yellow-50');
  });

  it('returns blue for low severity', () => {
    expect(getSeverityColor('low')).toBe('text-blue-600 bg-blue-50');
    expect(getSeverityColor('LOW')).toBe('text-blue-600 bg-blue-50');
  });

  it('returns gray for unknown severity', () => {
    expect(getSeverityColor('unknown')).toBe('text-gray-600 bg-gray-50');
    expect(getSeverityColor('')).toBe('text-gray-600 bg-gray-50');
    expect(getSeverityColor('invalid')).toBe('text-gray-600 bg-gray-50');
  });
});
