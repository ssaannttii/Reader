import { useEffect, useState } from 'react';
import { readDir, BaseDirectory } from '@tauri-apps/api/fs';
import { usePlayerStore } from '../state/playerStore';

interface VoiceOption {
  label: string;
  path: string;
}

function formatLabel(path: string): string {
  const parts = path.split(/\\|\//);
  const file = parts[parts.length - 1] ?? path;
  return file.replace('.onnx', '');
}

export default function Controls(): JSX.Element {
  const [voices, setVoices] = useState<VoiceOption[]>([]);
  const {
    voicePath,
    sentenceBreak,
    lengthScale,
    noiseScale,
    noiseW,
    updateSettings
  } = usePlayerStore();

  useEffect(() => {
    async function fetchVoices(): Promise<void> {
      try {
        const entries = await readDir('assets/voices/es_ES', {
          dir: BaseDirectory.Resource
        });
        const options: VoiceOption[] = [];
        entries.forEach((entry) => {
          if (entry.name?.endsWith('.onnx') && entry.path) {
            options.push({ label: formatLabel(entry.path), path: entry.path });
          }
        });
        setVoices(options);
        if (!voicePath && options.length > 0) {
          updateSettings({ voicePath: options[0]!.path });
        }
      } catch (error) {
        console.error('No se pudieron listar las voces', error);
      }
    }

    fetchVoices().catch(console.error);
  }, [updateSettings, voicePath]);

  return (
    <section className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-1" htmlFor="voice">
          Voz Piper
        </label>
        <select
          id="voice"
          className="w-full rounded border border-slate-300 bg-white text-slate-900 dark:bg-slate-800 dark:text-slate-100 p-2"
          value={voicePath}
          onChange={(event) => updateSettings({ voicePath: event.target.value })}
        >
          <option value="">Selecciona una voz</option>
          {voices.map((voice) => (
            <option key={voice.path} value={voice.path}>
              {voice.label}
            </option>
          ))}
        </select>
      </div>
      <Slider
        label="Pausa entre frases (ms)"
        min={200}
        max={1500}
        step={50}
        value={sentenceBreak}
        onChange={(value) => updateSettings({ sentenceBreak: value })}
      />
      <Slider
        label="Velocidad"
        min={0.5}
        max={2}
        step={0.1}
        value={lengthScale}
        onChange={(value) => updateSettings({ lengthScale: value })}
      />
      <Slider
        label="Ruido"
        min={0}
        max={1}
        step={0.1}
        value={noiseScale}
        onChange={(value) => updateSettings({ noiseScale: value })}
      />
      <Slider
        label="Claridad"
        min={0}
        max={1.5}
        step={0.1}
        value={noiseW}
        onChange={(value) => updateSettings({ noiseW: value })}
      />
    </section>
  );
}

interface SliderProps {
  label: string;
  min: number;
  max: number;
  step: number;
  value: number;
  onChange: (value: number) => void;
}

function Slider({ label, min, max, step, value, onChange }: SliderProps): JSX.Element {
  const display = step < 1 ? value.toFixed(2) : Math.round(value).toString();
  return (
    <div>
      <label className="flex justify-between text-sm font-medium mb-1">
        <span>{label}</span>
        <span>{display}</span>
      </label>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="w-full"
      />
    </div>
  );
}
