import React from 'react';
import type { LevelsCountBucket } from '../../types';
import { AlertCircle, AlertTriangle, Info } from 'lucide-react';
import { clsx } from 'clsx';

interface StatsOverviewProps {
  data: LevelsCountBucket | null;
  loading: boolean;
}

export const StatsOverview: React.FC<StatsOverviewProps> = ({ data, loading }) => {
  const stats = [
    {
      label: 'Errors',
      value: data?.error_count || 0,
      icon: AlertCircle,
      color: 'text-red-600',
      bgColor: 'bg-red-50',
      borderColor: 'border-red-200',
    },
    {
      label: 'Warnings',
      value: data?.warn_count || 0,
      icon: AlertTriangle,
      color: 'text-amber-600',
      bgColor: 'bg-amber-50',
      borderColor: 'border-amber-200',
    },
    {
      label: 'Info',
      value: data?.info_count || 0,
      icon: Info,
      color: 'text-blue-600',
      bgColor: 'bg-blue-50',
      borderColor: 'border-blue-200',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
      {stats.map((stat) => (
        <div
          key={stat.label}
          className={clsx(
            'p-6 rounded-lg border shadow-sm flex items-center',
            stat.bgColor,
            stat.borderColor
          )}
        >
          <div className={clsx('p-3 rounded-full bg-white bg-opacity-60 mr-4', stat.color)}>
            <stat.icon size={24} />
          </div>
          <div>
            <p className="text-sm font-medium text-gray-600 uppercase tracking-wide">
              {stat.label}
            </p>
            {loading ? (
              <div className="h-8 w-24 bg-gray-200 animate-pulse rounded mt-1"></div>
            ) : (
              <h3 className="text-3xl font-bold text-gray-900">{stat.value.toLocaleString()}</h3>
            )}
          </div>
        </div>
      ))}
    </div>
  );
};
