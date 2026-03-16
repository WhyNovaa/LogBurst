import { authFetch } from '../api/fetch';

interface ConsumeSseOptions<T> {
  signal?: AbortSignal;
  headers?: HeadersInit;
  onOpen?: () => void;
  onMessage: (data: T) => void;
  onParseError?: (error: unknown, rawData: string) => void;
  parse?: (rawData: string) => T;
}

const splitSseChunk = (chunk: string): string[] => chunk.split(/\r?\n\r?\n/);

const getEventData = (eventChunk: string): string | null => {
  const dataLines = eventChunk
    .split(/\r?\n/)
    .filter((line) => line.startsWith('data:'))
    .map((line) => line.slice(5).trimStart());

  if (dataLines.length === 0) {
    return null;
  }

  return dataLines.join('\n');
};

const emitMessage = <T>(
  eventChunk: string,
  options: ConsumeSseOptions<T>,
): void => {
  const rawData = getEventData(eventChunk);
  if (!rawData) {
    return;
  }

  const parse = options.parse ?? ((payload: string) => JSON.parse(payload) as T);

  try {
    options.onMessage(parse(rawData));
  } catch (error) {
    options.onParseError?.(error, rawData);
  }
};

export const consumeSse = async <T>(
  url: string,
  options: ConsumeSseOptions<T>,
): Promise<void> => {
  const headers = new Headers(options.headers);
  headers.set('Accept', 'text/event-stream');

  const response = await authFetch(url, {
    signal: options.signal,
    headers,
  });

  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error('SSE response body is not readable');
  }

  options.onOpen?.();

  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    const { value, done } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true });
    const chunks = splitSseChunk(buffer);
    buffer = chunks.pop() ?? '';

    for (const chunk of chunks) {
      emitMessage(chunk, options);
    }
  }

  buffer += decoder.decode();

  if (buffer.trim()) {
    emitMessage(buffer, options);
  }
};
