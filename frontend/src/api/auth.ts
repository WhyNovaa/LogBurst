import { apiClient } from './client';
import type { LoginRequest } from '../types';

export const login = async (credentials: LoginRequest): Promise<string> => {
  const response = await apiClient.post<string>('/auth', credentials);
  return response.data;
};
