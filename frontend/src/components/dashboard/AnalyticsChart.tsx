import React from 'react';
import {
  Bar,
  BarChart,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend
} from 'recharts';
import type { LevelsCountIntervalBucket } from '../../types';
import { format } from 'date-fns';
import { parseBackendDate } from '../../utils/dateUtils';

interface AnalyticsChartProps {
  data: LevelsCountIntervalBucket[];
  loading: boolean;
  timeRange?: string;
}

type ChartDataPoint = LevelsCountIntervalBucket & {
  timestamp: number;
};

export const AnalyticsChart: React.FC<AnalyticsChartProps> = ({ data, loading, timeRange = '24h' }) => {
  if (loading) {
    return (
      <div className="h-80 bg-white rounded-lg border border-beige-300 shadow-sm p-4 flex items-center justify-center">
        <div className="text-primary-400">Loading chart data...</div>
      </div>
    );
  }

  const formattedData: ChartDataPoint[] = data.map((item) => {
    const date = parseBackendDate(item.time_bucket);

    return {
      ...item,
      timestamp: date?.getTime() ?? 0,
      error_count: Number(item.error_count) || 0,
      warn_count: Number(item.warn_count) || 0,
      info_count: Number(item.info_count) || 0,
    };
  });

  const formatXAxis = (tick: number | string): string => {
      const timestamp = Number(tick);
      if (!Number.isFinite(timestamp) || timestamp === 0) return '';

      if (timeRange === '7d') {
          return format(new Date(timestamp), 'MMM dd');
      }

      return format(new Date(timestamp), 'HH:mm');
  };

  const formatTooltipLabel = (label: React.ReactNode): string => {
      if (typeof label === 'number' || typeof label === 'string') {
        const timestamp = Number(label);
        if (Number.isFinite(timestamp) && timestamp !== 0) {
          return format(new Date(timestamp), 'MMM dd, HH:mm');
        }
      }

      return 'Invalid Date';
  };

  const getTitle = () => {
      switch(timeRange) {
          case '1h': return 'Log Volume (Last Hour)';
          case '6h': return 'Log Volume (Last 6 Hours)';
          case '12h': return 'Log Volume (Last 12 Hours)';
          case '24h': return 'Log Volume (Last 24 Hours)';
          case '7d': return 'Log Volume (Last 7 Days)';
          default: return 'Log Volume Over Time';
      }
  }

  return (
    <div className="h-96 bg-white rounded-lg border border-beige-300 shadow-sm p-6 mb-6">
      <h3 className="text-lg font-bold text-primary-900 mb-4">{getTitle()}</h3>
      <div className="h-full w-full pb-6">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart
            data={formattedData}
            margin={{
              top: 10,
              right: 30,
              left: 10, // Added left margin for Y-axis labels
              bottom: 0,
            }}
          >
            <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
            <XAxis 
                dataKey="timestamp" 
                stroke="#6b7280" 
                fontSize={12} 
                tickFormatter={formatXAxis}
                minTickGap={30} // Prevent overcrowding
            />
            <YAxis 
                stroke="#6b7280" 
                fontSize={12} 
                width={40} // Explicit width to ensure labels are visible
            />
            <Tooltip
              contentStyle={{ backgroundColor: '#fff', borderColor: '#e5e7eb' }}
              labelStyle={{ color: '#374151' }}
              labelFormatter={formatTooltipLabel}
            />
            <Legend />
            <Bar
              dataKey="error_count"
              name="Errors"
              stackId="events"
              fill="#fecaca"
              stroke="#dc2626"
              barSize={24}
            />
            <Bar
              dataKey="warn_count"
              name="Warnings"
              stackId="events"
              fill="#fde68a"
              stroke="#d97706"
              barSize={24}
            />
            <Bar
              dataKey="info_count"
              name="Info"
              stackId="events"
              fill="#bfdbfe"
              stroke="#2563eb"
              radius={[4, 4, 0, 0]}
              barSize={24}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
};
