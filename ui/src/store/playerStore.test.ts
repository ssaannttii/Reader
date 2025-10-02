import type { Mock } from 'vitest';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn()
}));

let usePlayerStore: typeof import('./playerStore').usePlayerStore;
let defaultPreferences: typeof import('./playerStore').defaultPreferences;
let defaultVoices: typeof import('./playerStore').defaultVoices;

beforeAll(async () => {
  const store: Record<string, string> = {};
  const localStorageMock = {
    getItem: (key: string) => (key in store ? store[key] : null),
    setItem: (key: string, value: string) => {
      store[key] = String(value);
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      Object.keys(store).forEach((key) => delete store[key]);
    }
  };

  (globalThis as unknown as { window: typeof globalThis & { localStorage: typeof localStorageMock } }).window =
    { localStorage: localStorageMock } as typeof globalThis & { localStorage: typeof localStorageMock };
  (globalThis as unknown as { localStorage: typeof localStorageMock }).localStorage = localStorageMock;

  const module = await import('./playerStore');
  usePlayerStore = module.usePlayerStore;
  defaultPreferences = module.defaultPreferences;
  defaultVoices = module.defaultVoices;
});

const mockedInvoke = invoke as unknown as Mock;

beforeEach(() => {
  window.localStorage.clear();
  mockedInvoke.mockReset();
  usePlayerStore.setState({
    queue: [],
    currentIndex: 0,
    isPlaying: false,
    preferences: { ...defaultPreferences },
    availableVoices: defaultVoices
  });
});

afterEach(() => {
  usePlayerStore.setState({ queue: [], isPlaying: false, currentIndex: 0 });
});

const resolveInvoke = () => mockedInvoke.mockResolvedValue(undefined);

describe('playerStore', () => {
  it('filters paragraphs when setting queue', () => {
    usePlayerStore.getState().setQueue(['Hola', '', ' Mundo ']);
    const { queue, currentIndex } = usePlayerStore.getState();

    expect(queue).toEqual(['Hola', 'Mundo']);
    expect(currentIndex).toBe(0);
  });

  it('persists slider preferences to localStorage', async () => {
    usePlayerStore.getState().setRate(1.8);
    usePlayerStore.getState().setPitch(0.4);
    usePlayerStore.getState().setVolume(0.25);

    const persisted = window.localStorage.getItem('reader-player-preferences');
    const persistedString =
      typeof persisted === 'string' ? persisted : JSON.stringify(persisted ?? {});
    expect(persistedString).toMatch(/"rate":1.8/);
    expect(persistedString).toMatch(/"volume":0.25/);
    expect(persistedString).toMatch(/"pitch":0.5/);
  });

  it('invokes tauri speak command when playing from index', async () => {
    usePlayerStore.setState({ queue: ['Uno', 'Dos'] });
    resolveInvoke();
    await usePlayerStore.getState().playFrom(1);

    expect(mockedInvoke).toHaveBeenCalledWith(
      'speak',
      expect.objectContaining({
        request: expect.objectContaining({
          text: 'Dos',
          voice_id: 'default'
        })
      })
    );
    expect(usePlayerStore.getState().currentIndex).toBe(1);
    expect(usePlayerStore.getState().isPlaying).toBe(true);
  });

  it('stops audio when toggling from playing state', async () => {
    usePlayerStore.setState({ isPlaying: true });
    resolveInvoke();
    await usePlayerStore.getState().togglePlayPause();

    expect(mockedInvoke).toHaveBeenCalledWith('pause_audio');
    expect(usePlayerStore.getState().isPlaying).toBe(false);
  });
});
