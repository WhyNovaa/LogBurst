import React, { useEffect, useRef, useState } from 'react';
import type { Log } from '../../types';
import { getLiveLogsUrl } from '../../api/analytics';
import { clsx } from 'clsx';
import { Pause, Play, Terminal } from 'lucide-react';
import { format } from 'date-fns';
import { isAbortError } from '../../api/fetch';
import { parseBackendDate } from '../../utils/dateUtils';
import { consumeSse } from '../../utils/sse';

const MAX_LIVE_LOGS = 100;

export const LiveLogs: React.FC = () => {
  const [logs, setLogs] = useState<Log[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    if (isPaused) {
      return;
    }
    
    const abortController = new AbortController();
    const clearReconnectTimeout = () => {
      if (reconnectTimeoutRef.current !== null) {
        window.clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
    };

    const scheduleReconnect = () => {
      clearReconnectTimeout();
      reconnectTimeoutRef.current = window.setTimeout(() => {
        void connect();
      }, 3000);
    };

    const connect = async (): Promise<void> => {
      try {
        await consumeSse<Log>(getLiveLogsUrl(), {
          signal: abortController.signal,
          onOpen: () => setIsConnected(true),
          onMessage: (log) => {
            setLogs((previousLogs) => [...previousLogs, log].slice(-MAX_LIVE_LOGS));
          },
          onParseError: (error) => {
            console.error('Live logs parse error:', error);
          },
        });

        if (!abortController.signal.aborted) {
          setIsConnected(false);
          scheduleReconnect();
        }
      } catch (error: unknown) {
        if (isAbortError(error)) {
          return;
        }

        console.error('Live logs connection error:', error);
        setIsConnected(false);
        scheduleReconnect();
      }
    };

    void connect();

    return () => {
      abortController.abort();
      clearReconnectTimeout();
      setIsConnected(false);
    };
  }, [isPaused]);

  useEffect(() => {
    if (!isPaused) {
      logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, isPaused]);

  const togglePause = () => setIsPaused(!isPaused);
  const connectionIndicatorActive = isConnected && !isPaused;

  return (
    <div className="bg-neutral-900 rounded-lg shadow-lg overflow-hidden flex flex-col h-96 border border-neutral-700">
      <div className="bg-neutral-800 px-4 py-2 flex justify-between items-center border-b border-neutral-700">
        <div className="flex items-center gap-2">
          <Terminal size={16} className="text-primary-400" />
          <h3 className="text-neutral-200 font-mono text-sm font-bold">Live Logs</h3>
          <span className={clsx(
            "w-2 h-2 rounded-full ml-2",
            connectionIndicatorActive ? "bg-green-500 animate-pulse" : "bg-red-500"
          )} />
        </div>
        <button
          onClick={togglePause}
          className="text-neutral-400 hover:text-white transition-colors"
          title={isPaused ? "Resume" : "Pause"}
        >
          {isPaused ? <Play size={16} /> : <Pause size={16} />}
        </button>
      </div>
      
      <div className="flex-1 overflow-auto p-4 font-mono text-xs space-y-1 custom-scrollbar bg-[#0d1117]">
        {logs.length === 0 && (
          <div className="text-neutral-500 text-center mt-10 italic">Waiting for incoming logs...</div>
        )}
        {logs.map((log, index) => (
          <div key={index} className="flex gap-2 hover:bg-white/5 p-0.5 rounded">
            <span className="text-neutral-500 whitespace-nowrap">
              {(() => {
                const timestamp = parseBackendDate(log.timestamp);
                return timestamp ? format(timestamp, 'HH:mm:ss.SSS') : '--:--:--';
              })()}
            </span>
            <span className={clsx(
              "uppercase w-12 shrink-0 font-bold",
              log.level === 'error' && "text-red-400",
              log.level === 'warn' && "text-amber-400",
              log.level === 'info' && "text-blue-400",
            )}>
              {log.level}
            </span>
            <span className="text-purple-400 w-24 shrink-0 truncate" title={log.service}>
              {log.service}
            </span>
            <span className="text-neutral-300 break-all">
              {log.message}
            </span>
          </div>
        ))}
        <div ref={logsEndRef} />
      </div>
    </div>
  );
};
