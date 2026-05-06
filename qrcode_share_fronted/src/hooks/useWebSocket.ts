import {useCallback, useEffect, useRef} from 'react';
import {useConnectionStore} from '@/stores/connectionStore';
import {useMessageStore} from '@/stores/messageStore';
import {useSettingsStore} from '@/stores/settingsStore';
import type {WsServerMessage} from '@/types';
import {wsMessageToMessage} from '@/types/ws';

export function useWebSocket(channelId: string | null, password?: string) {
  const client = useConnectionStore((s) => s.client);
  const connect = useConnectionStore((s) => s.connect);
  const disconnect = useConnectionStore((s) => s.disconnect);
  const connectionState = useConnectionStore((s) => s.state);
  const subscriberCount = useConnectionStore((s) => s.subscriberCount);
  const setSubscriberCount = useConnectionStore((s) => s.setSubscriberCount);
  const addMessage = useMessageStore((s) => s.addMessage);

  const autoOpenRef = useRef(useSettingsStore.getState().autoOpenReceivedLinks);

  useEffect(() => {

    return useSettingsStore.subscribe((state) => {
      autoOpenRef.current = state.autoOpenReceivedLinks;
    });
  }, []);

  const channelIdRef = useRef(channelId);
  useEffect(() => {
    channelIdRef.current = channelId;
  });

  useEffect(() => {
    if (!channelId) return;
    connect(channelId, password);
    return () => {
      disconnect();
    };
  }, [channelId, connect, disconnect, password]);

  useEffect(() => {
    if (!client) return;




    return client.onMessage((message: WsServerMessage) => {
      switch (message.type) {
        case 'connected':
          setSubscriberCount(message.subscriber_count);
          break;
        case 'message': {
          const msg = wsMessageToMessage(message);
          const isNew = addMessage(msg);
          if (isNew && autoOpenRef.current && msg.link) {
            window.location.href = msg.link;
          }
          break;
        }
        case 'subscriber_update':
          setSubscriberCount(message.count);
          break;
        case 'pong':
          break;
        case 'error':
          console.error('[WS] Error from server:', message.code, message.message);
          break;
      }
    });
  }, [client, addMessage, setSubscriberCount]);

  const reconnect = useCallback(() => {
    if (channelIdRef.current) {
      connect(channelIdRef.current, password);
    }
  }, [connect, password]);

  return {
    connectionState,
    subscriberCount,
    reconnect,
  };
}
