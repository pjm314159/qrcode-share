import { create } from 'zustand';

interface SettingsState {
  autoOpenLinks: boolean;
  autoOpenReceivedLinks: boolean;
  setAutoOpenLinks: (value: boolean) => void;
  setAutoOpenReceivedLinks: (value: boolean) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  autoOpenLinks: localStorage.getItem('autoOpenLinks') === 'true',
  autoOpenReceivedLinks: localStorage.getItem('autoOpenReceivedLinks') === 'true',
  setAutoOpenLinks: (value: boolean) => {
    localStorage.setItem('autoOpenLinks', String(value));
    set({ autoOpenLinks: value });
  },
  setAutoOpenReceivedLinks: (value: boolean) => {
    localStorage.setItem('autoOpenReceivedLinks', String(value));
    set({ autoOpenReceivedLinks: value });
  },
}));
