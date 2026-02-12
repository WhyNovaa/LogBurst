import React, { useEffect, useState } from 'react';
import { StatsOverview } from '../components/dashboard/StatsOverview';
import { AnalyticsChart } from '../components/dashboard/AnalyticsChart';
import { LiveLogs } from '../components/dashboard/LiveLogs';
import { LogExplorer } from '../components/dashboard/LogExplorer';
import { getLevelsCount, getLevelsIntervals } from '../api/analytics';
import type { LevelsCountBucket, LevelsCountIntervalBucket } from '../types';
import { subHours } from 'date-fns';
import { RefreshCcw } from 'lucide-react';

export const Dashboard: React.FC = () => {
  const [stats, setStats] = useState<LevelsCountBucket | null>(null);
  const [chartData, setChartData] = useState<LevelsCountIntervalBucket[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchData = async () => {
    setLoading(true);
    try {
      const now = new Date();
      const from = subHours(now, 24); // Last 24 hours for chart

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
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 30000); // Refresh stats every 30s
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="space-y-6 pb-10">
      <div className="flex justify-between items-center mb-2">
        <h2 className="text-xl font-semibold text-primary-800">System Status</h2>
        <button 
            onClick={fetchData} 
            className="flex items-center gap-2 text-sm text-primary-600 hover:text-primary-800 transition"
        >
            <RefreshCcw size={14} className={loading ? "animate-spin" : ""} />
            Refresh
        </button>
      </div>

      <StatsOverview data={stats} loading={loading} />
      
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
            <AnalyticsChart data={chartData} loading={loading} />
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
