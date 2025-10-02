import { clsx } from 'clsx';

interface QueueListProps {
  items: string[];
  currentIndex: number;
  onSelect: (index: number) => void;
}

const QueueList = ({ items, currentIndex, onSelect }: QueueListProps) => (
  <ol className="space-y-2" aria-live="polite">
    {items.map((paragraph, index) => (
      <li key={index}>
        <button
          type="button"
          className={clsx(
            'w-full rounded-lg border px-4 py-3 text-left text-sm transition focus-ring',
            index === currentIndex
              ? 'border-primary bg-primary/10 text-primary'
              : 'border-transparent bg-muted/20 text-foreground hover:border-muted/60 hover:bg-muted/30'
          )}
          onClick={() => onSelect(index)}
        >
          <span className="block max-h-24 overflow-y-auto whitespace-pre-wrap">
            {paragraph.trim() || 'Párrafo vacío'}
          </span>
        </button>
      </li>
    ))}
  </ol>
);

export default QueueList;
