import { useMemo, useState } from 'react';
import Controls from '../components/Controls';
import ExportMenu from '../components/ExportMenu';
import { speak } from '../lib/tauri';
import { usePlayerStore } from '../state/playerStore';

export default function Reader(): JSX.Element {
  const {
    paragraphs,
    currentIndex,
    setCurrentIndex,
    nextParagraph,
    previousParagraph,
    voicePath,
    sentenceBreak,
    lengthScale,
    noiseScale,
    noiseW,
    setSynthesizing,
    isSynthesizing
  } = usePlayerStore();
  const [error, setError] = useState<string | null>(null);
  const [lastOutPath, setLastOutPath] = useState<string>('runtime/out.wav');

  const currentParagraph = paragraphs[currentIndex] ?? '';

  const paragraphCountWarning = useMemo(() => {
    if (paragraphs.length > 2000) {
      return 'Hay más de 2000 párrafos en cola. Considera dividir el documento.';
    }
    return null;
  }, [paragraphs.length]);

  const onSpeak = async (): Promise<void> => {
    if (!currentParagraph || !voicePath) {
      setError('Selecciona una voz y un párrafo.');
      return;
    }
    setSynthesizing(true);
    setError(null);
    try {
      const result = await speak({
        text: currentParagraph,
        voicePath,
        sentenceBreak,
        lengthScale,
        noiseScale,
        noiseW,
        playAfter: true,
        outPath: lastOutPath
      });
      if (!result.ok) {
        setError(result.message ?? 'Error en la síntesis.');
      } else if (result.outPath) {
        setLastOutPath(result.outPath);
        nextParagraph();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSynthesizing(false);
    }
  };

  return (
    <div className="grid gap-6 md:grid-cols-[2fr_1fr]">
      <section className="bg-white dark:bg-slate-800 rounded-lg shadow p-4 flex flex-col">
        <header className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-lg font-semibold">Lector</h2>
            {paragraphCountWarning && (
              <p className="text-xs text-amber-600">{paragraphCountWarning}</p>
            )}
          </div>
          <div className="space-x-2">
            <button
              type="button"
              onClick={previousParagraph}
              className="px-3 py-2 rounded bg-slate-200 dark:bg-slate-700"
              disabled={currentIndex === 0}
            >
              ⏮
            </button>
            <button
              type="button"
              onClick={onSpeak}
              className="px-3 py-2 rounded bg-blue-600 text-white disabled:opacity-50"
              disabled={isSynthesizing}
            >
              {isSynthesizing ? 'Sintetizando…' : '▶'}
            </button>
            <button
              type="button"
              onClick={nextParagraph}
              className="px-3 py-2 rounded bg-slate-200 dark:bg-slate-700"
            >
              ⏭
            </button>
          </div>
        </header>
        <div className="flex-1 overflow-y-auto space-y-3">
          {paragraphs.length === 0 ? (
            <p className="text-sm text-slate-500">Añade contenido desde la biblioteca.</p>
          ) : (
            paragraphs.map((paragraph, index) => (
              <article
                key={index}
                className={`rounded p-3 border ${
                  index === currentIndex
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30'
                    : 'border-transparent bg-slate-100 dark:bg-slate-900'
                }`}
                onClick={() => setCurrentIndex(index)}
                role="button"
                tabIndex={0}
                onKeyDown={(event) => {
                  if (event.key === 'Enter' || event.key === ' ') {
                    setCurrentIndex(index);
                  }
                }}
              >
                <p className="leading-relaxed">{paragraph}</p>
              </article>
            ))
          )}
        </div>
        {error && <p className="mt-3 text-sm text-red-500">{error}</p>}
        <div className="mt-4">
          <ExportMenu lastOutPath={lastOutPath} currentTitle={`parrafo-${currentIndex + 1}`} />
        </div>
      </section>
      <aside className="space-y-4">
        <Controls />
      </aside>
    </div>
  );
}
