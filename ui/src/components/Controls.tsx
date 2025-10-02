interface ControlsProps {
  isPlaying: boolean;
  onPlayPause: () => void;
  onNext: () => void;
  onExport: () => void;
  rate: number;
  pitch: number;
  volume: number;
  onRateChange: (value: number) => void;
  onPitchChange: (value: number) => void;
  onVolumeChange: (value: number) => void;
}

const sliderClass =
  'w-full accent-primary focus-ring cursor-pointer';

const Controls = ({
  isPlaying,
  onPlayPause,
  onNext,
  onExport,
  rate,
  pitch,
  volume,
  onRateChange,
  onPitchChange,
  onVolumeChange
}: ControlsProps) => (
  <section className="space-y-6 rounded-xl border border-muted/40 bg-background/60 p-4 shadow-sm">
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onPlayPause}
        className="rounded-md bg-primary px-4 py-2 text-sm font-semibold text-white focus-ring"
      >
        {isPlaying ? 'Pausa' : 'Reproducir'}
      </button>
      <button
        type="button"
        onClick={onNext}
        className="rounded-md border border-muted/40 px-4 py-2 text-sm font-medium text-foreground focus-ring"
      >
        Siguiente
      </button>
      <button
        type="button"
        onClick={onExport}
        className="rounded-md border border-muted/40 px-4 py-2 text-sm font-medium text-foreground focus-ring"
      >
        Exportar WAV
      </button>
    </div>

    <div className="grid gap-4 sm:grid-cols-3">
      <label className="flex flex-col gap-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted">Velocidad</span>
        <input
          aria-label="Velocidad"
          type="range"
          min="0.5"
          max="2"
          step="0.1"
          value={rate}
          onChange={(event) => onRateChange(Number(event.target.value))}
          className={sliderClass}
        />
        <span className="text-xs text-muted">{rate.toFixed(2)}x</span>
      </label>

      <label className="flex flex-col gap-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted">Tono</span>
        <input
          aria-label="Tono"
          type="range"
          min="0.5"
          max="2"
          step="0.1"
          value={pitch}
          onChange={(event) => onPitchChange(Number(event.target.value))}
          className={sliderClass}
        />
        <span className="text-xs text-muted">{pitch.toFixed(2)}x</span>
      </label>

      <label className="flex flex-col gap-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted">Volumen</span>
        <input
          aria-label="Volumen"
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={volume}
          onChange={(event) => onVolumeChange(Number(event.target.value))}
          className={sliderClass}
        />
        <span className="text-xs text-muted">{Math.round(volume * 100)}%</span>
      </label>
    </div>
  </section>
);

export default Controls;
