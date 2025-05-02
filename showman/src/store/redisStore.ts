import { create } from 'zustand';
import { WebSocketStatus } from '../hooks/useRedisWebSocket';

interface Message {
  channel: string;
  content: string;
  timestamp: number;
}

interface RedisStoreState {
  status: WebSocketStatus;
  connected: boolean;
  messages: Record<string, Message[]>;
  subscribedChannels: string[];
  
  // Actions
  setStatus: (status: WebSocketStatus) => void;
  addMessage: (channel: string, content: string) => void;
  addSubscribedChannel: (channel: string) => void;
  removeSubscribedChannel: (channel: string) => void;
  clearMessages: (channel?: string) => void;
}

export const useRedisStore = create<RedisStoreState>((set) => ({
  status: WebSocketStatus.CLOSED,
  connected: false,
  messages: {},
  subscribedChannels: [],
  
  setStatus: (status: WebSocketStatus) => set({
    status,
    connected: status === WebSocketStatus.OPEN,
  }),
  
  addMessage: (channel: string, content: string) => set((state) => {
    const channelMessages = state.messages[channel] || [];
    const newMessage: Message = {
      channel,
      content,
      timestamp: Date.now(),
    };
    
    return {
      messages: {
        ...state.messages,
        [channel]: [...channelMessages, newMessage],
      },
    };
  }),
  
  addSubscribedChannel: (channel: string) => set((state) => {
    if (state.subscribedChannels.includes(channel)) {
      return state;
    }
    return {
      subscribedChannels: [...state.subscribedChannels, channel],
    };
  }),
  
  removeSubscribedChannel: (channel: string) => set((state) => ({
    subscribedChannels: state.subscribedChannels.filter(ch => ch !== channel),
  })),
  
  clearMessages: (channel?: string) => set((state) => {
    if (channel) {
      // Using object destructuring but ignoring the extracted value
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { [channel]: ignored, ...restChannels } = state.messages;
      return {
        messages: restChannels,
      };
    }
    return {
      messages: {},
    };
  }),
})); 