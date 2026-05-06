import { apiClient } from './client';
import type {
  Channel,
  CreateChannelRequest,
  UpdateChannelRequest,
  ChannelListResponse,
  ApiResponse,
} from '@/types';

export async function createChannel(data: CreateChannelRequest): Promise<Channel> {
  const response = await apiClient.post<ApiResponse<Channel>>(
    '/api/channels',
    data
  );
  return response.data.data!;
}

export async function getChannel(
  channelId: string,
  password?: string
): Promise<Channel> {
  const headers: Record<string, string> = {};
  if (password) {
    headers['X-Channel-Password'] = password;
  }
  const response = await apiClient.get<ApiResponse<Channel>>(
    `/api/channels/${channelId}`,
    { headers }
  );
  return response.data.data!;
}

export async function listChannels(params?: {
  page?: number;
  limit?: number;
  channel_type?: string;
  search?: string;
}): Promise<ChannelListResponse> {
  const response = await apiClient.get<ApiResponse<ChannelListResponse>>(
    '/api/channels',
    { params }
  );
  return response.data.data!;
}

export async function updateChannel(
  channelId: string,
  data: UpdateChannelRequest
): Promise<Channel> {
  const response = await apiClient.patch<ApiResponse<Channel>>(
    `/api/channels/${channelId}`,
    data
  );
  return response.data.data!;
}

export async function deleteChannel(channelId: string): Promise<void> {
  await apiClient.delete(`/api/channels/${channelId}`);
}
