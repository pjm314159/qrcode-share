import { describe, it, expect } from 'vitest';
import { apiClient, API_BASE_URL } from '../client';

describe('API Client', () => {
  it('should create axios instance with correct base URL', () => {
    expect(API_BASE_URL).toBeDefined();
    expect(apiClient.defaults.baseURL).toBe(API_BASE_URL);
  });

  it('should have JSON content type header', () => {
    expect(apiClient.defaults.headers['Content-Type']).toBe('application/json');
  });

  it('should have timeout configured', () => {
    expect(apiClient.defaults.timeout).toBe(10000);
  });

  it('should have interceptors configured', () => {
    expect(apiClient.interceptors.response).toBeDefined();
    expect(apiClient.interceptors.response.handlers).toBeDefined();
    expect(apiClient.interceptors.response.handlers!.length).toBeGreaterThan(0);
  });
});
