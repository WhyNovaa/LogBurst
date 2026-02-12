import React, { useEffect, useRef, useState } from 'react';
import type { Log } from '../../types';
import { getLiveLogsUrl } from '../../api/analytics';
import { clsx } from 'clsx';
import { Pause, Play, Terminal } from 'lucide-react';
import { format } from 'date-fns';
import { parseBackendDate } from '../../utils/dateUtils';

export const LiveLogs: React.FC = () => {
  const [logs, setLogs] = useState<Log[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isPaused) return;
    
    const abortController = new AbortController();
    
    const connect = async () => {
      try {
        const token = localStorage.getItem('token');
        const response = await fetch(getLiveLogsUrl(), {
            headers: {
                Authorization: `Bearer ${token}`,
                Accept: 'text/event-stream',
            },
            signal: abortController.signal,
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        setIsConnected(true);
        
        const reader = response.body?.getReader();
        const decoder = new TextDecoder();
        
        if (!reader) return;

        while (true) {
            const { value, done } = await reader.read();
            if (done) break;
            
            const chunk = decoder.decode(value);
            const lines = chunk.split('\n\n');
            
            lines.forEach(line => {
                if (line.startsWith('data: ')) {
                    try {
                        const data = JSON.parse(line.substring(6));
                        setLogs(prev => [...prev, data].slice(-100)); // Keep last 100
                    } catch (e) {
                        // ignore parse error
                    }
                }
            });
        }
      } catch (err) {
        console.error('SSE Error:', err);
        setIsConnected(false);
        if (!abortController.signal.aborted) {
            setTimeout(connect, 3000); // Retry
        }
      }
    };

    connect();

    return () => {
        abortController.abort();
        setIsConnected(false);
    };
  }, [isPaused]);

  useEffect(() => {
    if (!isPaused) {
      logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, isPaused]);

  const togglePause = () => setIsPaused(!isPaused);

  return (
    <div className="bg-neutral-900 rounded-lg shadow-lg overflow-hidden flex flex-col h-96 border border-neutral-700">
      <div className="bg-neutral-800 px-4 py-2 flex justify-between items-center border-b border-neutral-700">
        <div className="flex items-center gap-2">
          <Terminal size={16} className="text-primary-400" />
          <h3 className="text-neutral-200 font-mono text-sm font-bold">Live Logs</h3>
          <span className={clsx(
            "w-2 h-2 rounded-full ml-2",
            isConnected ? "bg-green-500 animate-pulse" : "bg-red-500"
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
                try {
                  return format(parseBackendDate(log.timestamp), 'HH:mm:ss.SSS');
                } catch {
                  return '--:--:--';
                }
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
