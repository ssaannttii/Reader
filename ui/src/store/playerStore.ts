import { invoke } from '@tauri-apps/api/tauri';
import { create } from 'zustand';
import { persist, createJSONStorage, StateStorage } from 'zustand/middleware';

export type VoicePreference = {
  id: string;
  label: string;
};

export type ThemeMode = 'light' | 'dark';

export interface PlayerPreferences {
  voice: string;
  rate: number;
  pitch: number;
  volume: number;
  theme: ThemeMode;
}

export const defaultPreferences: PlayerPreferences = {
  voice: 'default',
  rate: 1,
  pitch: 1,
  volume: 1,
  theme: 'light'
};

export const defaultVoices: VoicePreference[] = [
  { id: 'default', label: 'Predeterminada' },
  { id: 'es-ES-carlfm-x-low', label: 'es-ES - CarlFM' }
];

export interface PlayerState {
  queue: string[];
  currentIndex: number;
  isPlaying: boolean;
  preferences: PlayerPreferences;
  availableVoices: VoicePreference[];
  setQueue: (paragraphs: string[]) => void;
  enqueue: (paragraph: string | string[]) => void;
  setCurrentIndex: (index: number) => void;
  setVoice: (voice: string) => void;
  setRate: (value: number) => void;
  setPitch: (value: number) => void;
  setVolume: (value: number) => void;
  toggleTheme: () => void;
  loadVoices: () => Promise<void>;
  playFrom: (index: number) => Promise<void>;
  togglePlayPause: () => Promise<void>;
  playNext: () => Promise<void>;
  importDocument: (command: string, args?: Record<string, unknown>) => Promise<void>;
}

const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

const noopStorage: StateStorage = {
  getItem: () => null,
  setItem: () => undefined,
  removeItem: () => undefined
};

const storage =
  typeof window !== 'undefined'
    ? createJSONStorage(() => window.localStorage)
    : createJSONStorage(() => noopStorage);

export const usePlayerStore = create<PlayerState>()(
  persist(
    (set, get) => ({
      queue: [],
      currentIndex: 0,
      isPlaying: false,
      preferences: { ...defaultPreferences },
      availableVoices: defaultVoices,
      setQueue: (paragraphs) => {
        set({ queue: paragraphs.filter((paragraph) => paragraph.trim().length > 0), currentIndex: 0 });
      },
      enqueue: (paragraph) => {
        const toAdd = Array.isArray(paragraph) ? paragraph : [paragraph];
        set((state) => ({ queue: [...state.queue, ...toAdd.filter((item) => item.trim().length > 0)] }));
      },
      setCurrentIndex: (index) => {
        set({ currentIndex: clamp(index, 0, Math.max(get().queue.length - 1, 0)) });
      },
      setVoice: (voice) => {
        set((state) => ({ preferences: { ...state.preferences, voice } }));
      },
      setRate: (value) => {
        set((state) => ({ preferences: { ...state.preferences, rate: Number(clamp(value, 0.5, 2).toFixed(2)) } }));
      },
      setPitch: (value) => {
        set((state) => ({ preferences: { ...state.preferences, pitch: Number(clamp(value, 0.5, 2).toFixed(2)) } }));
      },
      setVolume: (value) => {
        set((state) => ({ preferences: { ...state.preferences, volume: Number(clamp(value, 0, 1).toFixed(2)) } }));
      },
      toggleTheme: () => {
        set((state) => ({
          preferences: {
            ...state.preferences,
            theme: state.preferences.theme === 'light' ? 'dark' : 'light'
          }
        }));
      },
      loadVoices: async () => {
        try {
          const voices = await invoke<VoicePreference[]>('list_voices');
          if (Array.isArray(voices) && voices.length > 0) {
            set({ availableVoices: voices });
          }
        } catch (error) {
          console.warn('No se pudieron cargar las voces desde Tauri', error);
        }
      },
      playFrom: async (index) => {
        const state = get();
        const paragraph = state.queue[index];
        if (!paragraph) {
          return;
        }
        set({ currentIndex: index, isPlaying: true });
        try {
          await invoke('speak', {
            text: paragraph,
            options: {
              voice: state.preferences.voice,
              rate: state.preferences.rate,
              pitch: state.preferences.pitch,
              volume: state.preferences.volume
            }
          });
        } catch (error) {
          console.error('No se pudo reproducir el párrafo', error);
          set({ isPlaying: false });
        }
      },
      togglePlayPause: async () => {
        const { isPlaying } = get();
        try {
          if (isPlaying) {
            await invoke('stop_audio');
            set({ isPlaying: false });
          } else {
            await invoke('play_audio');
            set({ isPlaying: true });
          }
        } catch (error) {
          console.error('Error al alternar la reproducción', error);
        }
      },
      playNext: async () => {
        const { currentIndex, queue } = get();
        const nextIndex = currentIndex + 1;
        if (nextIndex < queue.length) {
          await get().playFrom(nextIndex);
        } else {
          await invoke('stop_audio');
          set({ isPlaying: false });
        }
      },
      importDocument: async (command, args = {}) => {
        try {
          const paragraphs = await invoke<string[]>(command, args);
          if (Array.isArray(paragraphs)) {
            get().setQueue(paragraphs);
          }
        } catch (error) {
          console.error('Error al importar documento', error);
        }
      }
    }),
    {
      name: 'reader-player-preferences',
      partialize: (state) => ({ preferences: state.preferences }),
      storage
    }
  )
);
