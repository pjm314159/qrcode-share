import type { Message } from './message';

export interface WsConnectedMessage {
  type: 'connected';
  channel_id: string;
  subscriber_count: number;
}

export interface WsMessageBroadcast {
  type: 'message';
  id: string;
  name: string;
  link: string;
  message_type?: string | null;
  location?: string | null;
  created_at: number;
  expire_at: number;
}

export interface WsSubscriberCountUpdate {
  type: 'subscriber_update';
  count: number;
}

export interface WsPong {
  type: 'pong';
}

export interface WsError {
  type: 'error';
  code: string;
  message: string;
}

export interface WsPing {
  type: 'ping';
}

export type WsServerMessage =
  | WsConnectedMessage
  | WsMessageBroadcast
  | WsSubscriberCountUpdate
  | WsPong
  | WsError;

export type WsClientMessage = WsPing;

export function wsMessageToMessage(msg: WsMessageBroadcast): Message {
  return {
    id: msg.id,
    name: msg.name,
    link: msg.link,
    link_domain: '',
    message_type: msg.message_type ?? undefined,
    location: msg.location ?? undefined,
    created_at: new Date(msg.created_at * 1000).toISOString(),
    expire_at: msg.expire_at ? new Date(msg.expire_at * 1000).toISOString() : '',
  };
}
