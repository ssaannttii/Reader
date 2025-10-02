import type { ChangeEventHandler } from 'react';
import type { VoicePreference } from '../store/playerStore';

interface VoiceSelectorProps {
  voices: VoicePreference[];
  value: string;
  onChange: ChangeEventHandler<HTMLSelectElement>;
}

const VoiceSelector = ({ voices, value, onChange }: VoiceSelectorProps) => (
  <label className="flex flex-col gap-2">
    <span className="text-sm font-medium text-muted">Voz</span>
    <select
      className="rounded-md border border-muted/50 bg-transparent px-3 py-2 text-sm focus-ring"
      value={value}
      onChange={onChange}
    >
      {voices.map((voice) => (
        <option key={voice.id} value={voice.id}>
          {voice.label}
        </option>
      ))}
    </select>
  </label>
);

export default VoiceSelector;
