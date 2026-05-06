import { apiClient } from './client';
import type {
  Message,
  CreateMessageRequest,
  MessageListResponse,
  ApiResponse,
} from '@/types';

export async function sendMessage(
  channelId: string,
  data: CreateMessageRequest,
  password?: string
): Promise<Message> {
  const headers: Record<string, string> = {};
  if (password) {
    headers['X-Channel-Password'] = password;
  }
  const response = await apiClient.post<ApiResponse<Message>>(
    `/api/channels/${channelId}/messages`,
    data,
    { headers }
  );
  return response.data.data!;
}

export async function getMessages(
  channelId: string,
  params?: {
    cursor?: string;
    limit?: number;
  },
  password?: string
): Promise<MessageListResponse> {
  const headers: Record<string, string> = {};
  if (password) {
    headers['X-Channel-Password'] = password;
  }
  const response = await apiClient.get<ApiResponse<MessageListResponse>>(
    `/api/channels/${channelId}/messages`,
    { params, headers }
  );
  return response.data.data!;
}

export async function getMessage(
  channelId: string,
  messageId: string,
  password?: string
): Promise<Message> {
  const headers: Record<string, string> = {};
  if (password) {
    headers['X-Channel-Password'] = password;
  }
  const response = await apiClient.get<ApiResponse<Message>>(
    `/api/channels/${channelId}/messages/${messageId}`,
    { headers }
  );
  return response.data.data!;
}
