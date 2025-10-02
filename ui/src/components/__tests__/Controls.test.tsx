import { fireEvent, render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import Controls from '../Controls';

const hasDocument = typeof document !== 'undefined';

(hasDocument ? describe : describe.skip)('Controls component', () => {
  const setup = () => {
    const onPlayPause = vi.fn();
    const onNext = vi.fn();
    const onExport = vi.fn();
    const onRateChange = vi.fn();
    const onPitchChange = vi.fn();
    const onVolumeChange = vi.fn();

    render(
      <Controls
        isPlaying={false}
        onPlayPause={onPlayPause}
        onNext={onNext}
        onExport={onExport}
        rate={1}
        pitch={1}
        volume={1}
        onRateChange={onRateChange}
        onPitchChange={onPitchChange}
        onVolumeChange={onVolumeChange}
      />
    );

    return { onPlayPause, onNext, onExport, onRateChange, onPitchChange, onVolumeChange };
  };

  it('triggers play and next callbacks', () => {
    const callbacks = setup();

    fireEvent.click(screen.getByRole('button', { name: /reproducir/i }));
    fireEvent.click(screen.getByRole('button', { name: /siguiente/i }));
    fireEvent.click(screen.getByRole('button', { name: /exportar/i }));

    expect(callbacks.onPlayPause).toHaveBeenCalledTimes(1);
    expect(callbacks.onNext).toHaveBeenCalledTimes(1);
    expect(callbacks.onExport).toHaveBeenCalledTimes(1);
  });

  it('reports slider changes', () => {
    const callbacks = setup();

    fireEvent.change(screen.getByLabelText(/velocidad/i), { target: { value: '1.5' } });
    fireEvent.change(screen.getByLabelText(/tono/i), { target: { value: '1.2' } });
    fireEvent.change(screen.getByLabelText(/volumen/i), { target: { value: '0.6' } });

    expect(callbacks.onRateChange).toHaveBeenCalledWith(1.5);
    expect(callbacks.onPitchChange).toHaveBeenCalledWith(1.2);
    expect(callbacks.onVolumeChange).toHaveBeenCalledWith(0.6);
  });
});
