import axios from 'axios';

const API_URL = '/v1';
const LOGIN_PATH = '/login';

export const apiClient = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

export const getAuthToken = (): string | null => localStorage.getItem('token');

export const clearAuthSession = (): void => {
  localStorage.removeItem('token');
};

export const redirectToLogin = (): void => {
  clearAuthSession();

  if (window.location.pathname !== LOGIN_PATH) {
    window.location.assign(LOGIN_PATH);
  }
};

export const buildAuthHeaders = (headers?: HeadersInit): Headers => {
  const authHeaders = new Headers(headers);
  const token = getAuthToken();

  if (token) {
    authHeaders.set('Authorization', `Bearer ${token}`);
  }

  return authHeaders;
};

apiClient.interceptors.request.use((config) => {
  const token = getAuthToken();
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401 && !error.config?.url?.endsWith('/auth')) {
      redirectToLogin();
    }
    return Promise.reject(error);
  }
);
