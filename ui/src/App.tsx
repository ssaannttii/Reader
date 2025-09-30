import { useState } from 'react';
import Library from './pages/Library';
import Reader from './pages/Reader';
import Dictionary from './pages/Dictionary';

const tabs = [
  { id: 'library', label: 'Biblioteca' },
  { id: 'reader', label: 'Lector' },
  { id: 'dictionary', label: 'Diccionario' }
] as const;

type TabId = (typeof tabs)[number]['id'];

export default function App(): JSX.Element {
  const [activeTab, setActiveTab] = useState<TabId>('library');

  return (
    <div className="min-h-screen bg-slate-100 dark:bg-slate-900 text-slate-900 dark:text-slate-100">
      <header className="flex items-center justify-between px-6 py-4 shadow bg-white dark:bg-slate-950">
        <h1 className="text-2xl font-semibold">Reader</h1>
        <nav className="space-x-2">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'bg-blue-600 text-white'
                  : 'bg-slate-200 dark:bg-slate-800 hover:bg-blue-500 hover:text-white'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </header>
      <main className="p-6">
        {activeTab === 'library' && <Library onOpenReader={() => setActiveTab('reader')} />}
        {activeTab === 'reader' && <Reader />}
        {activeTab === 'dictionary' && <Dictionary />}
      </main>
    </div>
  );
}
