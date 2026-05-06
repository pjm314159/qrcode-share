import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { WebSocketClient, type ConnectionState } from '../websocket';
import type { WsServerMessage } from '@/types';

describe('WebSocketClient', () => {
  let client: WebSocketClient;
  let mockWs: {
    readyState: number;
    send: ReturnType<typeof vi.fn>;
    close: ReturnType<typeof vi.fn>;
    onopen: ((ev: Event) => void) | null;
    onclose: (() => void) | null;
    onmessage: ((ev: MessageEvent) => void) | null;
    onerror: (() => void) | null;
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    mockWs = {
      readyState: WebSocket.CONNECTING,
      send: vi.fn(),
      close: vi.fn(),
      onopen: null,
      onclose: null,
      onmessage: null,
      onerror: null,
    };

    vi.spyOn(globalThis, 'WebSocket').mockImplementation(() => {
      return mockWs as unknown as WebSocket;
    });

    client = new WebSocketClient('test_channel');
  });

  afterEach(() => {
    client.disconnect();
    vi.useRealTimers();
  });

  describe('constructor', () => {
    it('should initialize with disconnected state', () => {
      expect(client.state).toBe('disconnected');
    });
  });

  describe('connect', () => {
    it('should transition to connecting then connected', async () => {
      const statePromise = new Promise<ConnectionState>((resolve) => {
        let resolved = false;
        client.onStateChange((state) => {
          if (state === 'connected' && !resolved) {
            resolved = true;
            resolve(state);
          }
        });
      });

      client.connect();

      mockWs.readyState = WebSocket.OPEN;
      mockWs.onopen?.({} as Event);

      const finalState = await statePromise;
      expect(finalState).toBe('connected');
    });

    it('should create WebSocket with correct URL pattern', () => {
      client.connect();
      const callArg = (WebSocket as unknown as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
      expect(callArg).toContain('/api/channels/test_channel/ws');
    });

    it('should include password in URL when provided', () => {
      const pwClient = new WebSocketClient('test_channel', 'secret');
      pwClient.connect();
      const callArg = (WebSocket as unknown as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
      expect(callArg).toContain('password=secret');
      pwClient.disconnect();
    });
  });

  describe('disconnect', () => {
    it('should set state to disconnected', () => {
      client.disconnect();
      expect(client.state).toBe('disconnected');
    });

    it('should close the WebSocket connection', () => {
      client.connect();
      client.disconnect();
      expect(mockWs.close).toHaveBeenCalled();
    });
  });

  describe('onMessage', () => {
    it('should call message handlers when a message is received', () => {
      const handler = vi.fn();
      client.onMessage(handler);
      client.connect();

      const testMessage: WsServerMessage = {
        type: 'connected',
        channel_id: 'test_channel',
        subscriber_count: 1,
      };

      mockWs.onmessage?.({
        data: JSON.stringify(testMessage),
      } as MessageEvent);

      expect(handler).toHaveBeenCalledWith(testMessage);
    });

    it('should return unsubscribe function', () => {
      const handler = vi.fn();
      const unsubscribe = client.onMessage(handler);
      client.connect();

      unsubscribe();

      mockWs.onmessage?.({
        data: JSON.stringify({ type: 'connected', channel_id: 'test_channel', subscriber_count: 1 }),
      } as MessageEvent);

      expect(handler).not.toHaveBeenCalled();
    });

    it('should ignore malformed messages', () => {
      const handler = vi.fn();
      client.onMessage(handler);
      client.connect();

      mockWs.onmessage?.({
        data: 'not json',
      } as MessageEvent);

      expect(handler).not.toHaveBeenCalled();
    });
  });

  describe('onStateChange', () => {
    it('should return unsubscribe function', () => {
      const handler = vi.fn();
      const unsubscribe = client.onStateChange(handler);

      unsubscribe();
      client.connect();

      expect(handler).not.toHaveBeenCalled();
    });
  });
});
