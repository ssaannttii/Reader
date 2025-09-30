import { invoke } from '@tauri-apps/api/core';
import type { PlayerSettings } from '../state/playerStore';

type SpeakPayload = PlayerSettings & {
  text: string;
  outPath?: string;
  playAfter?: boolean;
};

export interface SpeakResult {
  ok: boolean;
  outPath?: string;
  elapsedMs?: number;
  code?: string;
  message?: string;
}

export async function speak(payload: SpeakPayload): Promise<SpeakResult> {
  const response = await invoke<Record<string, unknown>>('speak', {
    text: payload.text,
    voicePath: payload.voicePath,
    outPath: payload.outPath,
    sentenceBreak: payload.sentenceBreak,
    lengthScale: payload.lengthScale,
    noiseScale: payload.noiseScale,
    noiseW: payload.noiseW,
    playAfter: payload.playAfter ?? false
  });
  return response as SpeakResult;
}

export async function importPdf(path: string): Promise<any> {
  return invoke('import_pdf', { path });
}

export async function importEpub(path: string): Promise<any> {
  return invoke('import_epub', { path });
}

export async function encodeToMp3(sourcePath: string, targetPath?: string): Promise<any> {
  return invoke('encode_audio', {
    sourcePath,
    targetPath,
    format: 'mp3'
  });
}

export async function listLexicon(): Promise<any> {
  return invoke('list_entries');
}

export async function upsertLexiconEntry(entry: {
  text: string;
  phonemes: string;
  caseSensitive: boolean;
}): Promise<any> {
  return invoke('upsert_entry', {
    text: entry.text,
    phonemes: entry.phonemes,
    caseSensitive: entry.caseSensitive
  });
}

export async function deleteLexiconEntry(text: string): Promise<any> {
  return invoke('delete_entry', { text });
}

export async function previewLexicon(sample: string): Promise<any> {
  return invoke('apply_preview', { sample });
}
