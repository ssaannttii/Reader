import { useState } from 'react';
import { encodeToMp3 } from '../lib/tauri';

interface ExportMenuProps {
  lastOutPath: string;
  currentTitle: string;
}

export default function ExportMenu({ lastOutPath, currentTitle }: ExportMenuProps): JSX.Element {
  const [status, setStatus] = useState<string | null>(null);

  const onExportMp3 = async (): Promise<void> => {
    setStatus('Exportando…');
    try {
      const target = `${lastOutPath.replace(/\.wav$/i, '')}-${currentTitle}.mp3`;
      const response = (await encodeToMp3(lastOutPath, target)) as any;
      if (!response.ok) {
        setStatus(response.message ?? 'ffmpeg no disponible.');
      } else {
        setStatus(`Exportado a ${response.path}`);
      }
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <div className="rounded border border-slate-300 dark:border-slate-700 p-3">
      <h3 className="text-sm font-semibold mb-2">Exportar</h3>
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onExportMp3}
          className="px-3 py-2 rounded bg-emerald-600 text-white"
        >
          Exportar MP3
        </button>
        {status && <p className="text-xs text-slate-500">{status}</p>}
      </div>
    </div>
  );
}
