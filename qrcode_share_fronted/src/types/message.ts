export interface Message {
  id: string;
  name: string;
  link: string;
  link_domain: string;
  message_type?: string;
  location?: string;
  expire_at: string;
  created_at: string;
}

export interface CreateMessageRequest {
  name: string;
  link: string;
  message_type?: string;
  location?: string;
  expire_seconds?: number;
}

export interface MessageListResponse {
  messages: Message[];
  has_more: boolean;
}

export interface MessageCard {
  name: string;
  expire_at: string;
  message_type?: string;
  link_domain: string;
}
