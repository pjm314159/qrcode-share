import { apiClient } from './client';

export interface HealthResponse {
  status: string;
  channels: number;
  connections: number;
  uptime_seconds: number;
}

export interface MetricsResponse {
  messages_sent: number;
  messages_received: number;
  active_connections: number;
  active_channels: number;
  channels: {
    total: number;
    with_password: number;
    without_password: number;
  };
}

export async function healthCheck(): Promise<HealthResponse> {
  const response = await apiClient.get<{ success: boolean; data: HealthResponse }>(
    '/health'
  );
  return response.data.data;
}

export async function getMetrics(): Promise<MetricsResponse> {
  const response = await apiClient.get<{ success: boolean; data: MetricsResponse }>(
    '/metrics'
  );
  return response.data.data;
}
