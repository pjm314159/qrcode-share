export interface Channel {
  id: string;
  name: string;
  has_password: boolean;
  link_limitation?: string[];
  channel_type?: string;
  location?: string;
  teacher?: string;
  created_at: string;
  subscriber_count: number;
  message_count: number;
}

export interface CreateChannelRequest {
  name: string;
  password?: string;
  link_limitation?: string[];
  channel_type?: string;
  location?: string;
  teacher?: string;
}

export interface UpdateChannelRequest {
  name?: string;
  password?: string;
  link_limitation?: string[];
  channel_type?: string;
  location?: string;
  teacher?: string;
}

export interface ChannelListResponse {
  channels: Channel[];
  total: number;
  page: number;
  limit: number;
}
