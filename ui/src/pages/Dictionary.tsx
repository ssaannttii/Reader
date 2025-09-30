import { useEffect, useState, type FormEvent } from 'react';
import { open, save } from '@tauri-apps/api/dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';
import {
  deleteLexiconEntry,
  listLexicon,
  previewLexicon,
  upsertLexiconEntry
} from '../lib/tauri';

interface LexiconEntry {
  text: string;
  phonemes: string;
  case_sensitive: boolean;
}

export default function Dictionary(): JSX.Element {
  const [entries, setEntries] = useState<LexiconEntry[]>([]);
  const [form, setForm] = useState({ text: '', phonemes: '', caseSensitive: false });
  const [preview, setPreview] = useState('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    refresh().catch(console.error);
  }, []);

  const refresh = async (): Promise<void> => {
    setError(null);
    const response = (await listLexicon()) as any;
    if (response.ok) {
      setEntries(response.entries ?? []);
    }
  };

  const onSubmit = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    if (!form.text || !form.phonemes) {
      setError('Completa texto y fonemas.');
      return;
    }
    setError(null);
    try {
      await saveEntry(form.text, form.phonemes, form.caseSensitive);
      setForm({ text: '', phonemes: '', caseSensitive: false });
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const onDelete = async (text: string): Promise<void> => {
    try {
      const response = (await deleteLexiconEntry(text)) as any;
      if (!response.ok) {
        setError(response.message ?? 'No se pudo eliminar.');
      } else {
        await refresh();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const onPreview = async (): Promise<void> => {
    if (!form.text) {
      setPreview('');
      return;
    }
    const sample = `Prueba con ${form.text}`;
    const response = (await previewLexicon(sample)) as any;
    if (response.ok) {
      setPreview(response.transformed ?? sample);
    }
  };

  const onExport = async (): Promise<void> => {
    const target = await save({ defaultPath: 'lexicon.json' });
    if (!target) {
      return;
    }
    const payload = JSON.stringify({ entries }, null, 2);
    await writeTextFile(target, payload);
  };

  const onExportCsv = async (): Promise<void> => {
    const target = await save({ defaultPath: 'lexicon.csv' });
    if (!target) {
      return;
    }
    const csv = ['texto,fonemas,case_sensitive', ...entries.map((entry) => `${entry.text},${entry.phonemes},${entry.case_sensitive}`)].join('\n');
    await writeTextFile(target, csv);
  };

  const onImport = async (): Promise<void> => {
    try {
      const filePath = await open({ filters: [{ name: 'Lexicon', extensions: ['json', 'csv'] }] });
      if (!filePath || Array.isArray(filePath)) {
        return;
      }
      const content = await readTextFile(filePath);
      if (filePath.endsWith('.json')) {
        const json = JSON.parse(content);
        const imported: LexiconEntry[] = json.entries ?? [];
        for (const entry of imported) {
          await saveEntry(entry.text, entry.phonemes, entry.case_sensitive);
        }
        await refresh();
      } else {
        const rows = content.split('\n').slice(1);
        const parsed = rows
          .filter(Boolean)
          .map((row) => {
            const [text, phonemes, flag] = row.split(',');
            return {
              text,
              phonemes,
              case_sensitive: flag === 'true'
            };
          });
        for (const entry of parsed) {
          await saveEntry(entry.text, entry.phonemes, entry.case_sensitive);
        }
        await refresh();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const saveEntry = async (text: string, phonemes: string, caseSensitive: boolean): Promise<void> => {
    const response = (await upsertLexiconEntry({
      text,
      phonemes,
      caseSensitive
    })) as any;
    if (!response.ok) {
      throw new Error(response.message ?? 'No se pudo guardar.');
    }
  };

  return (
    <div className="grid gap-4 md:grid-cols-[1fr_1fr]">
      <section className="bg-white dark:bg-slate-800 rounded-lg p-4 shadow">
        <h2 className="text-lg font-semibold mb-3">Editar diccionario</h2>
        <form className="space-y-3" onSubmit={onSubmit}>
          <div>
            <label className="block text-sm font-medium mb-1" htmlFor="text">
              Texto
            </label>
            <input
              id="text"
              value={form.text}
              onChange={(event) => setForm((prev) => ({ ...prev, text: event.target.value }))}
              className="w-full rounded border border-slate-300 bg-white dark:bg-slate-900 dark:border-slate-700 p-2"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1" htmlFor="phonemes">
              Fonemas
            </label>
            <input
              id="phonemes"
              value={form.phonemes}
              onChange={(event) => setForm((prev) => ({ ...prev, phonemes: event.target.value }))}
              className="w-full rounded border border-slate-300 bg-white dark:bg-slate-900 dark:border-slate-700 p-2"
            />
          </div>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={form.caseSensitive}
              onChange={(event) => setForm((prev) => ({ ...prev, caseSensitive: event.target.checked }))}
            />
            Sensible a mayúsculas/minúsculas
          </label>
          <div className="flex gap-2">
            <button type="submit" className="px-4 py-2 rounded bg-blue-600 text-white">
              Guardar
            </button>
            <button type="button" onClick={onPreview} className="px-4 py-2 rounded bg-slate-200 dark:bg-slate-700">
              Previsualizar
            </button>
          </div>
          {error && <p className="text-sm text-red-500">{error}</p>}
          {preview && <p className="text-sm text-slate-500">{preview}</p>}
        </form>
        <div className="mt-4 flex gap-2 text-sm">
          <button type="button" onClick={onExport} className="px-3 py-2 rounded bg-emerald-600 text-white">
            Exportar JSON
          </button>
          <button type="button" onClick={onExportCsv} className="px-3 py-2 rounded bg-emerald-600 text-white">
            Exportar CSV
          </button>
          <button type="button" onClick={onImport} className="px-3 py-2 rounded bg-slate-200 dark:bg-slate-700">
            Importar
          </button>
        </div>
      </section>
      <section className="bg-white dark:bg-slate-800 rounded-lg p-4 shadow">
        <h2 className="text-lg font-semibold mb-3">Entradas ({entries.length})</h2>
        <ul className="space-y-2 max-h-96 overflow-y-auto">
          {entries.map((entry) => (
            <li key={entry.text} className="border border-slate-200 dark:border-slate-700 rounded p-2 flex justify-between">
              <div>
                <p className="font-medium">{entry.text}</p>
                <p className="text-xs text-slate-500">{entry.phonemes}</p>
              </div>
              <button
                type="button"
                onClick={() => onDelete(entry.text)}
                className="text-xs text-red-500"
              >
                Eliminar
              </button>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
