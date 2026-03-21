export type TimestampValue =
  | string
  | number
  | {
      timestamp: string | number;
    };

export interface User {
  username: string;
  token: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface Log {
  timestamp: TimestampValue;
  level: 'info' | 'warn' | 'error';
  service: string;
  message: string;
  raw_data: string;
}

export interface LevelsCountBucket {
  error_count: number;
  warn_count: number;
  info_count: number;
}

export interface LevelsCountIntervalBucket {
  time_bucket: TimestampValue;
  error_count: number;
  warn_count: number;
  info_count: number;
}

export interface IntervalQuery {
  from: string;
  to: string;
}

export interface LogsFilter extends IntervalQuery {
  service?: string;
  level?: Log['level'] | '';
  limit: number;
}
