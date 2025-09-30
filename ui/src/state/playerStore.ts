import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface PlayerSettings {
  voicePath: string;
  sentenceBreak: number;
  lengthScale: number;
  noiseScale: number;
  noiseW: number;
}

export interface PlayerState extends PlayerSettings {
  paragraphs: string[];
  currentIndex: number;
  isSynthesizing: boolean;
  isPlaying: boolean;
  setParagraphs: (items: string[]) => void;
  nextParagraph: () => void;
  previousParagraph: () => void;
  setCurrentIndex: (index: number) => void;
  setSynthesizing: (value: boolean) => void;
  setPlaying: (value: boolean) => void;
  updateSettings: (settings: Partial<PlayerSettings>) => void;
}

const defaultSettings: PlayerSettings = {
  voicePath: '',
  sentenceBreak: 550,
  lengthScale: 1.0,
  noiseScale: 0.5,
  noiseW: 0.9
};

export const usePlayerStore = create<PlayerState>()(
  persist(
    (set, get) => ({
      ...defaultSettings,
      paragraphs: [],
      currentIndex: 0,
      isSynthesizing: false,
      isPlaying: false,
      setParagraphs: (items) =>
        set({
          paragraphs: items,
          currentIndex: 0
        }),
      nextParagraph: () => {
        const { currentIndex, paragraphs } = get();
        if (currentIndex + 1 < paragraphs.length) {
          set({ currentIndex: currentIndex + 1 });
        }
      },
      previousParagraph: () => {
        const { currentIndex } = get();
        if (currentIndex > 0) {
          set({ currentIndex: currentIndex - 1 });
        }
      },
      setCurrentIndex: (index) => set({ currentIndex: index }),
      setSynthesizing: (value) => set({ isSynthesizing: value }),
      setPlaying: (value) => set({ isPlaying: value }),
      updateSettings: (settings) => set(settings)
    }),
    {
      name: 'reader-player-settings',
      partialize: ({ voicePath, sentenceBreak, lengthScale, noiseScale, noiseW }) => ({
        voicePath,
        sentenceBreak,
        lengthScale,
        noiseScale,
        noiseW
      })
    }
  )
);
