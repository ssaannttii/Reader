import { invoke } from '@tauri-apps/api/tauri';
import { create } from 'zustand';
import { persist, createJSONStorage, StateStorage } from 'zustand/middleware';

export type VoicePreference = {
  id: string;
  label: string;
  language?: string;
  quality?: string;
};

type BackendVoice = {
  id: string;
  label: string;
  language?: string | null;
  quality?: string | null;
};

type SpeakResponse = {
  output_path: string;
  duration_ms: number;
  stderr?: string | null;
  playback_id?: number | null;
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
  importDocument: (command: string, path: string) => Promise<void>;
}

const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

const noopStorage: StateStorage = {
  getItem: () => null,
  setItem: () => undefined,
  removeItem: () => undefined
};

const storage = createJSONStorage(() => {
  if (typeof window !== 'undefined' && window?.localStorage) {
    return window.localStorage;
  }
  return noopStorage;
});

export const usePlayerStore = create<PlayerState>()(
  persist(
    (set, get) => ({
      queue: [],
      currentIndex: 0,
      isPlaying: false,
      preferences: { ...defaultPreferences },
      availableVoices: defaultVoices,
      setQueue: (paragraphs) => {
        const cleaned = paragraphs
          .map((paragraph) => paragraph.trim())
          .filter((paragraph) => paragraph.length > 0);
        set({ queue: cleaned, currentIndex: 0 });
      },
      enqueue: (paragraph) => {
        const toAdd = (Array.isArray(paragraph) ? paragraph : [paragraph])
          .map((item) => item.trim())
          .filter((item) => item.length > 0);
        if (toAdd.length === 0) {
          return;
        }
        set((state) => ({ queue: [...state.queue, ...toAdd] }));
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
          const voices = await invoke<BackendVoice[]>('list_voices');
          if (Array.isArray(voices) && voices.length > 0) {
            set({
              availableVoices: voices.map((voice) => ({
                id: voice.id,
                label: voice.label,
                language: voice.language ?? undefined,
                quality: voice.quality ?? undefined
              }))
            });
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
          const lengthScale = Number((1 / state.preferences.rate).toFixed(2));
          await invoke<SpeakResponse>('speak', {
            request: {
              text: paragraph,
              voice_id: state.preferences.voice,
              length_scale: lengthScale,
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
            await invoke('pause_audio');
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
      importDocument: async (command, path) => {
        try {
          const paragraphs = await invoke<string[]>(command, { path });
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
