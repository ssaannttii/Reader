import { useEffect, useMemo, useState } from 'react';
import Controls from '../components/Controls';
import QueueList from '../components/QueueList';
import VoiceSelector from '../components/VoiceSelector';
import { usePlayerStore } from '../store/playerStore';

const parseParagraphs = (content: string) =>
  content
    .split(/\n{2,}/)
    .map((paragraph) => paragraph.trim())
    .filter((paragraph) => paragraph.length > 0);

const Reader = () => {
  const queue = usePlayerStore((state) => state.queue);
  const currentIndex = usePlayerStore((state) => state.currentIndex);
  const isPlaying = usePlayerStore((state) => state.isPlaying);
  const preferences = usePlayerStore((state) => state.preferences);
  const availableVoices = usePlayerStore((state) => state.availableVoices);
  const setQueue = usePlayerStore((state) => state.setQueue);
  const setVoice = usePlayerStore((state) => state.setVoice);
  const setRate = usePlayerStore((state) => state.setRate);
  const setPitch = usePlayerStore((state) => state.setPitch);
  const setVolume = usePlayerStore((state) => state.setVolume);
  const playFrom = usePlayerStore((state) => state.playFrom);
  const playNext = usePlayerStore((state) => state.playNext);
  const togglePlayPause = usePlayerStore((state) => state.togglePlayPause);
  const importDocument = usePlayerStore((state) => state.importDocument);
  const toggleTheme = usePlayerStore((state) => state.toggleTheme);
  const loadVoices = usePlayerStore((state) => state.loadVoices);
  const [editorValue, setEditorValue] = useState('');
  const [importing, setImporting] = useState(false);

  useEffect(() => {
    setEditorValue(queue.join('\n\n'));
  }, [queue]);

  useEffect(() => {
    void loadVoices();
  }, [loadVoices]);

  const highlightedParagraphs = useMemo(() => queue, [queue]);

  const handleApplyText = () => {
    const paragraphs = parseParagraphs(editorValue);
    setQueue(paragraphs);
  };

  const handlePlayPause = async () => {
    if (!isPlaying) {
      if (queue.length === 0) {
        const paragraphs = parseParagraphs(editorValue);
        if (paragraphs.length === 0) {
          return;
        }
        setQueue(paragraphs);
        await playFrom(0);
        return;
      }
      await playFrom(currentIndex);
    } else {
      await togglePlayPause();
    }
  };

  const handleNext = async () => {
    if (queue.length === 0) {
      return;
    }
    await playNext();
  };

  const handleImport = async (command: string) => {
    setImporting(true);
    try {
      await importDocument(command);
    } finally {
      setImporting(false);
    }
  };

  return (
    <main className="min-h-screen bg-background px-4 py-8 text-foreground">
      <div className="mx-auto flex max-w-6xl flex-col gap-8">
        <header className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h1 className="text-3xl font-semibold">Lector asistido</h1>
            <p className="text-sm text-muted">
              Prepara tus párrafos, organízalos en la cola y controla la reproducción de Piper desde Tauri.
            </p>
          </div>
          <button
            type="button"
            onClick={toggleTheme}
            className="self-start rounded-md border border-muted/40 px-3 py-2 text-sm font-medium focus-ring"
            aria-pressed={preferences.theme === 'dark'}
          >
            {preferences.theme === 'dark' ? 'Tema claro' : 'Tema oscuro'}
          </button>
        </header>

        <section className="grid gap-8 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)]">
          <div className="space-y-6">
            <div className="rounded-xl border border-muted/40 bg-background/70 p-4 shadow-sm">
              <div className="flex items-center justify-between gap-4">
                <h2 className="text-lg font-semibold">Editor de párrafos</h2>
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={() => handleImport('import_epub')}
                    className="rounded-md border border-muted/40 px-3 py-2 text-xs font-medium focus-ring"
                    disabled={importing}
                  >
                    Importar EPUB
                  </button>
                  <button
                    type="button"
                    onClick={() => handleImport('import_pdf')}
                    className="rounded-md border border-muted/40 px-3 py-2 text-xs font-medium focus-ring"
                    disabled={importing}
                  >
                    Importar PDF
                  </button>
                  <button
                    type="button"
                    onClick={() => handleImport('import_text')}
                    className="rounded-md border border-muted/40 px-3 py-2 text-xs font-medium focus-ring"
                    disabled={importing}
                  >
                    Importar texto
                  </button>
                </div>
              </div>
              <textarea
                className="mt-4 h-64 w-full resize-y rounded-lg border border-muted/40 bg-background/80 p-4 text-sm leading-relaxed focus-ring"
                placeholder="Escribe o pega tus párrafos. Se separarán por dobles saltos de línea."
                value={editorValue}
                onChange={(event) => setEditorValue(event.target.value)}
              />
              <div className="mt-4 flex justify-end">
                <button
                  type="button"
                  onClick={handleApplyText}
                  className="rounded-md bg-primary px-4 py-2 text-sm font-semibold text-white focus-ring"
                >
                  Actualizar cola
                </button>
              </div>
            </div>

            <section className="space-y-4">
              <h2 className="text-lg font-semibold">Reproductor</h2>
              <VoiceSelector
                voices={availableVoices}
                value={preferences.voice}
                onChange={(event) => setVoice(event.target.value)}
              />
              <Controls
                isPlaying={isPlaying}
                onPlayPause={handlePlayPause}
                onNext={handleNext}
                rate={preferences.rate}
                pitch={preferences.pitch}
                volume={preferences.volume}
                onRateChange={setRate}
                onPitchChange={setPitch}
                onVolumeChange={setVolume}
              />
            </section>
          </div>

          <aside className="space-y-6">
            <section className="rounded-xl border border-muted/40 bg-background/70 p-4 shadow-sm">
              <h2 className="text-lg font-semibold">Cola de lectura</h2>
              {queue.length === 0 ? (
                <p className="mt-4 text-sm text-muted">Añade contenido para comenzar a escuchar.</p>
              ) : (
                <QueueList items={queue} currentIndex={currentIndex} onSelect={(index) => void playFrom(index)} />
              )}
            </section>

            <section className="rounded-xl border border-muted/40 bg-background/70 p-4 shadow-sm">
              <h2 className="text-lg font-semibold">Vista previa</h2>
              <div className="mt-4 space-y-4">
                {highlightedParagraphs.map((paragraph, index) => (
                  <article
                    key={index}
                    className={`rounded-lg border p-4 text-sm leading-relaxed ${
                      index === currentIndex
                        ? 'border-primary bg-primary/10 text-primary'
                        : 'border-transparent bg-muted/10 text-foreground'
                    }`}
                  >
                    {paragraph}
                  </article>
                ))}
              </div>
            </section>
          </aside>
        </section>
      </div>
    </main>
  );
};

export default Reader;
