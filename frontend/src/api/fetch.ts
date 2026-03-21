import { buildAuthHeaders, redirectToLogin } from './client';

export class FetchError extends Error {
  public readonly status: number;

  constructor(
    message: string,
    status: number,
  ) {
    super(message);
    this.name = 'FetchError';
    this.status = status;
  }
}

export const isAbortError = (error: unknown): error is DOMException =>
  error instanceof DOMException && error.name === 'AbortError';

export const authFetch = async (
  input: RequestInfo | URL,
  init: RequestInit = {},
): Promise<Response> => {
  const response = await fetch(input, {
    ...init,
    headers: buildAuthHeaders(init.headers),
  });

  if (response.status === 401) {
    redirectToLogin();
    throw new FetchError('Unauthorized', response.status);
  }

  if (!response.ok) {
    throw new FetchError(`Request failed with status ${response.status}`, response.status);
  }

  return response;
};
