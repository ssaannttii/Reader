import { act } from '@testing-library/react';
import type { Mock } from 'vitest';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';
import {
  usePlayerStore,
  defaultPreferences,
  defaultVoices
} from './playerStore';

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn()
}));

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

const resolveInvoke = () => mockedInvoke.mockResolvedValueOnce(undefined);

describe('playerStore', () => {
  it('filters paragraphs when setting queue', () => {
    usePlayerStore.getState().setQueue(['Hola', '', ' Mundo ']);
    const { queue, currentIndex } = usePlayerStore.getState();

    expect(queue).toEqual(['Hola', 'Mundo']);
    expect(currentIndex).toBe(0);
  });

  it('persists slider preferences to localStorage', async () => {
    await act(async () => {
      usePlayerStore.getState().setRate(1.8);
      usePlayerStore.getState().setPitch(0.4);
      usePlayerStore.getState().setVolume(0.25);
    });

    const persisted = window.localStorage.getItem('reader-player-preferences');
    expect(persisted).toMatch(/"rate":1.8/);
    expect(persisted).toMatch(/"volume":0.25/);
    expect(persisted).toMatch(/"pitch":0.5/);
  });

  it('invokes tauri speak command when playing from index', async () => {
    usePlayerStore.setState({ queue: ['Uno', 'Dos'] });
    resolveInvoke();

    await act(async () => {
      await usePlayerStore.getState().playFrom(1);
    });

    expect(mockedInvoke).toHaveBeenCalledWith(
      'speak',
      expect.objectContaining({
        text: 'Dos',
        options: expect.objectContaining({ voice: 'default' })
      })
    );
    expect(usePlayerStore.getState().currentIndex).toBe(1);
    expect(usePlayerStore.getState().isPlaying).toBe(true);
  });

  it('stops audio when toggling from playing state', async () => {
    usePlayerStore.setState({ isPlaying: true });
    resolveInvoke();

    await act(async () => {
      await usePlayerStore.getState().togglePlayPause();
    });

    expect(mockedInvoke).toHaveBeenCalledWith('stop_audio');
    expect(usePlayerStore.getState().isPlaying).toBe(false);
  });
});
