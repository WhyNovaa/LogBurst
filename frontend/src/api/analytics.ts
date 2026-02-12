import { apiClient } from './client';
import type { LevelsCountBucket, LevelsCountIntervalBucket, IntervalQuery } from '../types';

export const getLevelsCount = async (): Promise<LevelsCountBucket> => {
  const response = await apiClient.get<LevelsCountBucket>('/analytics/levels/count');
  return response.data;
};

export const getLevelsIntervals = async (params: IntervalQuery): Promise<LevelsCountIntervalBucket[]> => {
  const response = await apiClient.get<LevelsCountIntervalBucket[]>('/analytics/levels/intervals', { params });
  return response.data;
};

export const getLiveLogsUrl = () => '/v1/analytics/last';

export const getHistoricalLogsUrl = (params: any) => {
    const searchParams = new URLSearchParams();
    searchParams.append('from', params.from);
    searchParams.append('to', params.to);
    if (params.service) searchParams.append('service', params.service);
    if (params.level) searchParams.append('level', params.level);
    
    return `/v1/analytics/logs/interval?${searchParams.toString()}`;
}
