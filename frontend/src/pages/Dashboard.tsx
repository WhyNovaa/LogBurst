import React, { useCallback, useEffect, useState } from 'react';
import { StatsOverview } from '../components/dashboard/StatsOverview';
import { AnalyticsChart } from '../components/dashboard/AnalyticsChart';
import { LiveLogs } from '../components/dashboard/LiveLogs';
import { LogExplorer } from '../components/dashboard/LogExplorer';
import { getLevelsCount, getLevelsIntervals } from '../api/analytics';
import type { LevelsCountBucket, LevelsCountIntervalBucket } from '../types';
import { subHours, subDays } from 'date-fns';
import { RefreshCcw, Calendar } from 'lucide-react';

type TimeRange = '1h' | '6h' | '12h' | '24h' | '7d';

const getFromDate = (range: TimeRange): Date => {
  const now = new Date();
  switch (range) {
    case '1h': return subHours(now, 1);
    case '6h': return subHours(now, 6);
    case '12h': return subHours(now, 12);
    case '24h': return subHours(now, 24);
    case '7d': return subDays(now, 7);
    default: return subHours(now, 24);
  }
};

export const Dashboard: React.FC = () => {
  const [stats, setStats] = useState<LevelsCountBucket | null>(null);
  const [chartData, setChartData] = useState<LevelsCountIntervalBucket[]>([]);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState<TimeRange>('24h');

  const fetchData = useCallback(async () => {
    setLoading(true);
    try {
      const now = new Date();
      const from = getFromDate(timeRange);

      const [statsData, chartRes] = await Promise.all([
        getLevelsCount(),
        getLevelsIntervals({ from: from.toISOString(), to: now.toISOString() })
      ]);

      setStats(statsData);
      setChartData(chartRes);
    } catch (error) {
      console.error('Failed to fetch dashboard data:', error);
    } finally {
      setLoading(false);
    }
  }, [timeRange]);

  useEffect(() => {
    void fetchData();
  }, [fetchData]);

  useEffect(() => {
    const interval = window.setInterval(() => {
      void fetchData();
    }, 30000);

    return () => window.clearInterval(interval);
  }, [fetchData]);

  return (
    <div className="space-y-6 pb-10">
      <div className="flex justify-between items-center mb-2">
        <h2 className="text-xl font-semibold text-primary-800">System Status</h2>
        
        <div className="flex items-center gap-4">
            <div className="flex items-center bg-white rounded-md shadow-sm border border-beige-300 px-3 py-1.5">
                <Calendar size={16} className="text-primary-500 mr-2" />
                <select 
                    value={timeRange} 
                    onChange={(e) => setTimeRange(e.target.value as TimeRange)}
                    className="bg-transparent border-none text-sm text-primary-700 focus:ring-0 cursor-pointer outline-none"
                >
                    <option value="1h">Last 1 Hour</option>
                    <option value="6h">Last 6 Hours</option>
                    <option value="12h">Last 12 Hours</option>
                    <option value="24h">Last 24 Hours</option>
                    <option value="7d">Last 7 Days</option>
                </select>
            </div>

            <button 
                onClick={() => void fetchData()} 
                className="flex items-center gap-2 text-sm text-primary-600 hover:text-primary-800 transition"
            >
                <RefreshCcw size={14} className={loading ? "animate-spin" : ""} />
                Refresh
            </button>
        </div>
      </div>

      <StatsOverview data={stats} loading={loading} />
      
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
            <AnalyticsChart data={chartData} loading={loading} timeRange={timeRange} />
        </div>
        <div className="lg:col-span-1">
             <LiveLogs />
        </div>
      </div>

      <div className="mt-8">
        <LogExplorer />
      </div>
    </div>
  );
};
