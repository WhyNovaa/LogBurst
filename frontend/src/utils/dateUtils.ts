import type { TimestampValue } from '../types';

const parseTimestampNumber = (value: number): Date => {
  if (value > 1e9 && value < 1e12) {
    return new Date(value * 1000);
  }

  return new Date(value);
};

export const parseBackendDate = (
  dateInput: TimestampValue | null | undefined,
): Date | null => {
  if (dateInput == null || dateInput === '') {
    return null;
  }

  if (typeof dateInput === 'object' && 'timestamp' in dateInput) {
    return parseBackendDate(dateInput.timestamp);
  }

  if (typeof dateInput === 'number') {
    const date = parseTimestampNumber(dateInput);
    return Number.isNaN(date.getTime()) ? null : date;
  }

  if (/^\d+$/.test(dateInput)) {
    const date = parseTimestampNumber(Number(dateInput));
    return Number.isNaN(date.getTime()) ? null : date;
  }

  let date = new Date(dateInput);
  if (!Number.isNaN(date.getTime())) {
    return date;
  }

  let isoLike = dateInput.replace(/^(\d{4}-\d{2}-\d{2})\s(\d{2}:\d{2})/, '$1T$2');
  isoLike = isoLike.replace(/\s+([+-]\d)/, '$1');
  isoLike = isoLike.replace(/([+-]\d{2}:\d{2}):\d{2}$/, '$1');

  date = new Date(isoLike);
  if (!Number.isNaN(date.getTime())) {
    return date;
  }

  return null;
};
