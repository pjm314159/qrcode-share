import axios from 'axios';
import type { ApiResponse } from '@/types';

const API_BASE_URL = import.meta.env.VITE_API_URL || '';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response) {
      const apiResponse = error.response.data as ApiResponse<unknown>;
      if (apiResponse && !apiResponse.success) {
        return Promise.reject({
          code: apiResponse.error?.code || 'UNKNOWN_ERROR',
          message: apiResponse.error?.message || 'An unexpected error occurred',
          retry_after_seconds: apiResponse.error?.retry_after_seconds,
          status: error.response.status,
        });
      }
    }
    return Promise.reject({
      code: 'NETWORK_ERROR',
      message: error.message || 'Network error occurred',
      status: 0,
    });
  }
);

export { apiClient, API_BASE_URL };
