import React from 'react';
import {
  AreaChart,
  Area,
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
}

export const AnalyticsChart: React.FC<AnalyticsChartProps> = ({ data, loading }) => {
  if (loading) {
    return (
      <div className="h-80 bg-white rounded-lg border border-beige-300 shadow-sm p-4 flex items-center justify-center">
        <div className="text-primary-400">Loading chart data...</div>
      </div>
    );
  }

  const formattedData = data.map(item => {
    try {
        const date = parseBackendDate(item.time_bucket);
        return {
            ...item,
            // Use timestamp (number) as unique key for X-axis to avoid merging duplicate HH:mm strings
            timestamp: date.getTime(),
            // Ensure counts are numbers
            error_count: Number(item.error_count),
            warn_count: Number(item.warn_count),
            info_count: Number(item.info_count),
        };
    } catch (e) {
        return {
            ...item,
            timestamp: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
        };
    }
  });

  const formatXAxis = (tick: any) => {
      if (!tick) return '';
      // If it's a timestamp
      if (typeof tick === 'number') {
           return format(new Date(tick), 'HH:mm');
      }
      return String(tick);
  };

  const formatTooltipLabel = (label: any) => {
      if (typeof label === 'number' && label !== 0) {
        return format(new Date(label), 'MMM dd, HH:mm');
      }
      return 'Invalid Date';
  };

  return (
    <div className="h-96 bg-white rounded-lg border border-beige-300 shadow-sm p-6 mb-6">
      <h3 className="text-lg font-bold text-primary-900 mb-4">Log Volume Over Time</h3>
      <div className="h-full w-full pb-6">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
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
            <Area
              type="monotone"
              dataKey="error_count"
              name="Errors"
              stackId="1"
              stroke="#dc2626"
              fill="#fecaca"
            />
            <Area
              type="monotone"
              dataKey="warn_count"
              name="Warnings"
              stackId="1"
              stroke="#d97706"
              fill="#fde68a"
            />
            <Area
              type="monotone"
              dataKey="info_count"
              name="Info"
              stackId="1"
              stroke="#2563eb"
              fill="#bfdbfe"
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
};
