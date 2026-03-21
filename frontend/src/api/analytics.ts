import { apiClient } from './client';
import type {
  IntervalQuery,
  LevelsCountBucket,
  LevelsCountIntervalBucket,
  LogsFilter,
} from '../types';

export const getLevelsCount = async (): Promise<LevelsCountBucket> => {
  const response = await apiClient.get<LevelsCountBucket>('/analytics/levels/count');
  return response.data;
};

export const getLevelsIntervals = async (params: IntervalQuery): Promise<LevelsCountIntervalBucket[]> => {
  const response = await apiClient.get<LevelsCountIntervalBucket[]>('/analytics/levels/intervals', { params });
  return response.data;
};

export const getServices = async (): Promise<string[]> => {
  const response = await apiClient.get<string[]>('/analytics/services');
  return response.data;
};

export const getLiveLogsUrl = () => '/v1/analytics/last';

export const getHistoricalLogsUrl = (params: LogsFilter): string => {
  const searchParams = new URLSearchParams({
    from: params.from,
    to: params.to,
    limit: params.limit.toString(),
  });

  const service = params.service?.trim();
  if (service) {
    searchParams.append('service', service);
  }

  if (params.level) {
    searchParams.append('level', params.level);
  }

  return `/v1/analytics/logs/interval?${searchParams.toString()}`;
};
