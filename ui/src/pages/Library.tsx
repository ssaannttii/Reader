import { useCallback, useState } from 'react';
import { open } from '@tauri-apps/api/dialog';
import { readTextFile } from '@tauri-apps/api/fs';
import { importPdf, importEpub } from '../lib/tauri';
import { usePlayerStore } from '../state/playerStore';

interface LibraryProps {
  onOpenReader: () => void;
}

interface PreviewItem {
  title: string;
  paragraphs: string[];
}

const MAX_PARAGRAPHS = 2000;

export default function Library({ onOpenReader }: LibraryProps): JSX.Element {
  const setParagraphs = usePlayerStore((state) => state.setParagraphs);
  const [preview, setPreview] = useState<PreviewItem | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setLoading] = useState(false);

  const handleFile = useCallback(
    async (filePath: string) => {
      setLoading(true);
      setError(null);
      try {
        if (filePath.endsWith('.pdf')) {
          const response = (await importPdf(filePath)) as any;
          if (!response.ok) {
            throw new Error(response.message ?? 'No se pudo importar PDF');
          }
          const pages: string[] = response.pages?.map((page: { text: string }) => page.text) ?? [];
          setPreview({
            title: response.meta?.title ?? 'PDF',
            paragraphs: pages.flatMap((page) => page.split('\n\n'))
          });
        } else if (filePath.endsWith('.epub')) {
          const response = (await importEpub(filePath)) as any;
          if (!response.ok) {
            throw new Error(response.message ?? 'No se pudo importar EPUB');
          }
          const chapters = response.chapters ?? [];
          setPreview({
            title: chapters[0]?.title ?? 'EPUB',
            paragraphs: chapters.flatMap((chapter: any) => chapter.paragraphs ?? [])
          });
        } else if (filePath.endsWith('.txt')) {
          const content = await readTextFile(filePath);
          if (!content) {
            throw new Error('No se pudo leer el archivo TXT');
          }
          setPreview({ title: filePath.split(/\\|\//).pop() ?? 'TXT', paragraphs: content.split('\n') });
        } else {
          throw new Error('Formato no soportado');
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setPreview(null);
      } finally {
        setLoading(false);
      }
    },
    [setPreview]
  );

  const onImportClick = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'Documentos', extensions: ['pdf', 'epub', 'txt'] }
      ]
    });
    if (typeof selected === 'string') {
      await handleFile(selected);
    }
  }, [handleFile]);

  const onAddToReader = useCallback(() => {
    if (!preview) {
      return;
    }
    if (preview.paragraphs.length > MAX_PARAGRAPHS) {
      setError(`El documento supera el máximo permitido de ${MAX_PARAGRAPHS} párrafos.`);
      return;
    }
    setParagraphs(preview.paragraphs);
    onOpenReader();
  }, [preview, onOpenReader, setParagraphs]);

  return (
    <div className="grid gap-4 md:grid-cols-2">
      <section
        className="border-2 border-dashed border-slate-300 dark:border-slate-700 rounded-lg p-6 flex flex-col items-center justify-center text-center"
        onDragOver={(event) => {
          event.preventDefault();
        }}
        onDrop={(event) => {
          event.preventDefault();
          const file = event.dataTransfer?.files?.[0];
          if (file) {
            handleFile(file.path).catch(console.error);
          }
        }}
      >
        <p className="text-lg font-medium">Importa tu documento</p>
        <p className="text-sm text-slate-500 dark:text-slate-400 mt-2">
          Arrastra y suelta un PDF, EPUB o TXT, o usa el botón.
        </p>
        <button
          type="button"
          onClick={onImportClick}
          className="mt-4 px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700"
        >
          Importar
        </button>
        {isLoading && <p className="mt-4 text-sm">Procesando…</p>}
        {error && <p className="mt-4 text-sm text-red-500">{error}</p>}
      </section>
      <section className="bg-white dark:bg-slate-800 rounded-lg p-4 shadow">
        <h2 className="text-lg font-semibold">Vista previa</h2>
        {preview ? (
          <div className="mt-3 space-y-2 max-h-96 overflow-y-auto pr-2">
            <p className="font-medium">{preview.title}</p>
            {preview.paragraphs.slice(0, 10).map((paragraph, index) => (
              <p key={index} className="text-sm leading-relaxed">
                {paragraph}
              </p>
            ))}
            {preview.paragraphs.length > 10 && <p className="text-xs text-slate-500">…</p>}
          </div>
        ) : (
          <p className="text-sm text-slate-500 mt-2">Importa un documento para previsualizarlo.</p>
        )}
        <button
          type="button"
          onClick={onAddToReader}
          disabled={!preview}
          className="mt-4 px-4 py-2 rounded bg-emerald-600 text-white disabled:opacity-50"
        >
          Añadir al Lector
        </button>
      </section>
    </div>
  );
}
