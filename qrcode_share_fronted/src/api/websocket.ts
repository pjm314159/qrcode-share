import type { WsServerMessage, WsClientMessage } from '@/types';
import { getWsBaseUrl } from '@/utils/helpers';

const WS_BASE_URL = import.meta.env.VITE_WS_URL || getWsBaseUrl();
const HEARTBEAT_INTERVAL = 30_000;
const MAX_RECONNECT_DELAY = 30_000;
const BASE_RECONNECT_DELAY = 1_000;

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting';

export type WsMessageHandler = (message: WsServerMessage) => void;
export type WsStateHandler = (state: ConnectionState) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private readonly channelId: string;
  private readonly password: string | undefined;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempts = 0;
  private messageHandlers = new Set<WsMessageHandler>();
  private stateHandlers = new Set<WsStateHandler>();
  private _state: ConnectionState = 'disconnected';
  private disposed = false;

  constructor(channelId: string, password?: string) {
    this.channelId = channelId;
    this.password = password;
  }

  get state(): ConnectionState {
    return this._state;
  }

  private setState(state: ConnectionState): void {
    this._state = state;
    this.stateHandlers.forEach((handler) => handler(state));
  }

  connect(): void {
    if (this.disposed) return;

    this.setState('connecting');
    const params = new URLSearchParams();
    if (this.password) {
      params.set('password', this.password);
    }
    const url = `${WS_BASE_URL}/api/channels/${this.channelId}/ws${params.toString() ? '?' + params.toString() : ''}`;

    try {
      this.ws = new WebSocket(url);
    } catch (err) {
      console.error('[WS] Failed to create WebSocket:', err);
      this.scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      this.setState('connected');
      this.reconnectAttempts = 0;
      this.startHeartbeat();
    };

    this.ws.onmessage = (event) => {
      try {
        const message: WsServerMessage = JSON.parse(event.data);
        this.messageHandlers.forEach((handler) => handler(message));
      } catch (err) {
        console.warn('[WS] Failed to parse message:', err);
      }
    };

    this.ws.onclose = () => {
      this.stopHeartbeat();
      if (!this.disposed) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = () => {
      console.error('[WS] Connection error');
    };
  }

  disconnect(): void {
    this.disposed = true;
    this.stopHeartbeat();
    this.clearReconnectTimer();
    if (this.ws) {
      this.ws.onclose = null;
      this.ws.onerror = null;
      this.ws.onmessage = null;
      this.ws.onopen = null;
      this.ws.close();
      this.ws = null;
    }
    this.setState('disconnected');
  }

  onMessage(handler: WsMessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => {
      this.messageHandlers.delete(handler);
    };
  }

  onStateChange(handler: WsStateHandler): () => void {
    this.stateHandlers.add(handler);
    return () => {
      this.stateHandlers.delete(handler);
    };
  }

  private send(message: WsClientMessage): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      this.send({ type: 'ping' });
    }, HEARTBEAT_INTERVAL);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  private scheduleReconnect(): void {
    this.setState('reconnecting');
    const delay = Math.min(
      BASE_RECONNECT_DELAY * Math.pow(2, this.reconnectAttempts),
      MAX_RECONNECT_DELAY
    );
    this.reconnectAttempts++;
    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}
