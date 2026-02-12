import React, { useState, useRef, useEffect, useCallback } from 'react';
import type { Log, LogsFilter } from '../../types';
import { getHistoricalLogsUrl } from '../../api/analytics';
import { clsx } from 'clsx';
import { Search, X, AlertTriangle, Loader2 } from 'lucide-react';
import { format, subHours } from 'date-fns';
import { parseBackendDate } from '../../utils/dateUtils';

const MAX_LOGS = 5000;
const BATCH_SIZE = 50; // Render this many logs at a time

export const LogExplorer: React.FC = () => {
  // distinct states for "all fetched logs" vs "what is rendered"
  const allLogsRef = useRef<Log[]>([]);
  const [visibleLogs, setVisibleLogs] = useState<Log[]>([]);
  
  const [loading, setLoading] = useState(false);
  const [limitReached, setLimitReached] = useState(false);
  
  const [filters, setFilters] = useState<LogsFilter>({
    from: subHours(new Date(), 1).toISOString(),
    to: new Date().toISOString(),
    service: '',
    level: '',
  });

  const abortControllerRef = useRef<AbortController | null>(null);
  const observerTarget = useRef<HTMLDivElement>(null);

  // Function to load more logs from the ref to the state
  const loadMoreLogs = useCallback(() => {
    setVisibleLogs(current => {
      const currentLength = current.length;
      const totalLength = allLogsRef.current.length;
      
      if (currentLength >= totalLength) return current;
      
      const nextBatch = allLogsRef.current.slice(currentLength, currentLength + BATCH_SIZE);
      return [...current, ...nextBatch];
    });
  }, []);

  // Intersection Observer for infinite scrolling
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          loadMoreLogs();
        }
      },
      { threshold: 0.1 }
    );

    if (observerTarget.current) {
      observer.observe(observerTarget.current);
    }

    return () => observer.disconnect();
  }, [loadMoreLogs]);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    // Reset everything
    allLogsRef.current = [];
    setVisibleLogs([]);
    setLimitReached(false);
    setLoading(true);
    
    const abortController = new AbortController();
    abortControllerRef.current = abortController;

    try {
      const token = localStorage.getItem('token');
      const queryParams = {
        ...filters,
        from: new Date(filters.from).toISOString(),
        to: new Date(filters.to).toISOString()
      };
      
      const url = getHistoricalLogsUrl(queryParams);
      
      const response = await fetch(url, {
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: 'text/event-stream',
        },
        signal: abortController.signal,
      });

      if (!response.ok) throw new Error('Failed to fetch logs');

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();
      
      if (!reader) return;

      let totalLogs = 0;

      while (true) {
        const { value, done } = await reader.read();
        if (done) {
            break;
        }
        
        const chunk = decoder.decode(value);
        const lines = chunk.split('\n\n');
        
        let newLogsCount = 0;
        
        lines.forEach(line => {
          if (line.startsWith('data: ')) {
            try {
              const data = JSON.parse(line.substring(6));
              allLogsRef.current.push(data);
              newLogsCount++;
              totalLogs++;
            } catch (e) {
               // ignore
            }
          }
        });

        // If we have very few logs visible, auto-fill the screen immediately
        // so the user sees something without scrolling
        if (visibleLogs.length < BATCH_SIZE && newLogsCount > 0) {
            loadMoreLogs();
        }

        if (totalLogs >= MAX_LOGS) {
            setLimitReached(true);
            abortController.abort();
            break;
        }
      }
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        console.error('Search error:', err);
      }
    } finally {
      setLoading(false);
    }
  };

  const clearResults = () => {
      if (abortControllerRef.current) {
          abortControllerRef.current.abort();
      }
      allLogsRef.current = [];
      setVisibleLogs([]);
      setLimitReached(false);
      setLoading(false);
  }

  return (
    <div className="bg-white rounded-lg border border-beige-300 shadow-sm flex flex-col h-[600px]">
      <div className="p-4 border-b border-beige-200 bg-beige-50 rounded-t-lg">
        <h3 className="text-lg font-bold text-primary-900 mb-4">Log Explorer</h3>
        <form onSubmit={handleSearch} className="grid grid-cols-1 md:grid-cols-5 gap-4 items-end">
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">From</label>
            <input
              type="datetime-local"
              className="w-full text-sm border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500 p-2 border"
              value={format(new Date(filters.from), "yyyy-MM-dd'T'HH:mm")}
              onChange={e => setFilters({...filters, from: new Date(e.target.value).toISOString()})}
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">To</label>
            <input
              type="datetime-local"
              className="w-full text-sm border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500 p-2 border"
              value={format(new Date(filters.to), "yyyy-MM-dd'T'HH:mm")}
              onChange={e => setFilters({...filters, to: new Date(e.target.value).toISOString()})}
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">Service</label>
            <input
              type="text"
              className="w-full text-sm border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500 p-2 border"
              placeholder="e.g. api-service"
              value={filters.service}
              onChange={e => setFilters({...filters, service: e.target.value})}
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1">Level</label>
            <select
              className="w-full text-sm border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500 p-2 border bg-white"
              value={filters.level}
              onChange={e => setFilters({...filters, level: e.target.value})}
            >
              <option value="">All Levels</option>
              <option value="error">Error</option>
              <option value="warn">Warn</option>
              <option value="info">Info</option>
            </select>
          </div>
          <div className="flex gap-2">
            <button
              type="submit"
              disabled={loading}
              className="flex-1 bg-primary-600 text-white p-2 rounded-md hover:bg-primary-700 transition flex items-center justify-center gap-2 text-sm font-medium"
            >
              {loading ? (
                  <Loader2 size={16} className="animate-spin" />
              ) : (
                  <Search size={16} />
              )}
              Search
            </button>
            {(visibleLogs.length > 0 || loading) && (
                <button
                    type="button"
                    onClick={clearResults}
                    className="px-3 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 transition"
                >
                    <X size={16} />
                </button>
            )}
          </div>
        </form>
      </div>

      <div className="flex-1 overflow-auto bg-gray-50 p-4 relative">
        {visibleLogs.length === 0 && !loading && (
            <div className="h-full flex items-center justify-center text-gray-400 italic">
                No logs found. Adjust filters and search.
            </div>
        )}

        {limitReached && (
             <div className="sticky top-0 z-10 mb-4 bg-amber-50 border border-amber-200 text-amber-800 px-4 py-3 rounded-md flex items-center gap-2 text-sm shadow-sm">
                 <AlertTriangle size={16} />
                 <span>
                     Limit of {MAX_LOGS} logs reached. Please narrow your time range.
                 </span>
             </div>
        )}
        
        <div className="space-y-2">
            {visibleLogs.map((log, i) => (
                <div key={i} className="bg-white p-3 rounded border border-gray-200 text-sm shadow-sm hover:shadow-md transition-shadow">
                    <div className="flex justify-between items-start mb-1">
                        <div className="flex gap-2 items-center">
                            <span className={clsx(
                                "px-2 py-0.5 rounded text-xs font-bold uppercase",
                                log.level === 'error' && "bg-red-100 text-red-700",
                                log.level === 'warn' && "bg-amber-100 text-amber-700",
                                log.level === 'info' && "bg-blue-100 text-blue-700",
                            )}>
                                {log.level}
                            </span>
                            <span className="font-mono text-xs text-gray-500">
                                {(() => {
                                    try {
                                        return format(parseBackendDate(log.timestamp), 'yyyy-MM-dd HH:mm:ss.SSS');
                                    } catch {
                                        return 'Invalid Date';
                                    }
                                })()}
                            </span>
                            <span className="bg-purple-50 text-purple-700 px-2 py-0.5 rounded text-xs border border-purple-100">
                                {log.service}
                            </span>
                        </div>
                    </div>
                    <div className="font-mono text-gray-800 whitespace-pre-wrap break-all">
                        {log.message}
                    </div>
                    {log.raw_data && log.raw_data !== "{}" && log.raw_data !== "" && (
                         <div className="mt-2 text-xs text-gray-500 font-mono bg-gray-50 p-2 rounded overflow-x-auto">
                            {log.raw_data}
                         </div>
                    )}
                </div>
            ))}
            
            {/* Sentinel element for infinite scrolling */}
            <div ref={observerTarget} className="h-4 w-full flex items-center justify-center text-xs text-gray-400">
                {loading ? 'Streaming...' : (visibleLogs.length < allLogsRef.current.length ? 'Loading more...' : '')}
            </div>
        </div>
      </div>
      <div className="p-2 border-t border-gray-200 bg-white text-xs text-gray-500 text-center flex justify-between px-4">
          <span>Showing {visibleLogs.length} of {allLogsRef.current.length} loaded</span>
          {limitReached && <span className="text-amber-600 font-medium">Limit Reached</span>}
      </div>
    </div>
  );
};
