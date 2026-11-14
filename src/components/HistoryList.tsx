import {
  ChevronRight,
  Clipboard,
  Copy,
  Image as ImageIcon,
  X,
  Zap,
} from 'lucide-react';

import type { RefObject } from 'react';

import type { Messages } from '../i18n';
import { excerpt, getEntryKind } from '../lib/entries';
import type { ClipEntry, EntryKind } from '../types';

interface HistoryListProps {
  entries: ClipEntry[];
  selectedIndex: number;
  query: string;
  monitoring: boolean;
  busy: boolean;
  t: Messages;
  listRef: RefObject<HTMLDivElement | null>;
  formatTime: (timestampMs: number) => string;
  onSelect: (index: number) => void;
  onPaste: (id: number) => void;
  onRemove: (id: number) => void;
}

function kindLabel(kind: EntryKind, t: Messages): string {
  if (kind === 'image') return t.image;
  if (kind === 'link') return t.link;
  if (kind === 'code') return t.code;
  return t.text;
}

export function HistoryList({
  entries,
  selectedIndex,
  query,
  monitoring,
  busy,
  t,
  listRef,
  formatTime,
  onSelect,
  onPaste,
  onRemove,
}: HistoryListProps) {
  return (
    <div
      className="entries"
      ref={listRef}
      role="listbox"
      aria-label={t.subtitle}
    >
      {entries.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon">
            <Clipboard size={26} />
          </div>

          <h2>{query ? t.noMatches : t.waiting}</h2>

          <p>
            {query
              ? t.noMatchesDetail
              : monitoring
                ? t.waitingDetail
                : t.pausedDetail}
          </p>
        </div>
      ) : (
        entries.map((entry, index) => {
          const kind = getEntryKind(entry);
          const selected = index === selectedIndex;

          return (
            <div
              key={entry.id}
              className={`entry ${selected ? 'selected' : ''}`}
              data-selected={selected}
              role="option"
              aria-selected={selected}
              onMouseEnter={() => onSelect(index)}
            >
              <button
                type="button"
                className="entry-select"
                onClick={() => onPaste(entry.id)}
                disabled={busy}
                title={t.clickPaste}
              >
                <div className={`entry-icon ${kind}`}>
                  {kind === 'image' ? (
                    <ImageIcon size={16} />
                  ) : kind === 'link' ? (
                    <ChevronRight size={16} />
                  ) : kind === 'code' ? (
                    <Zap size={15} />
                  ) : (
                    <Copy size={15} />
                  )}
                </div>

                <div className="entry-content">
                  {entry.kind === 'image' ? (
                    <div className="image-row">
                      {entry.thumbnail && (
                        <img
                          className="entry-thumbnail"
                          src={entry.thumbnail}
                          alt={t.imagePreview}
                        />
                      )}

                      <div className="entry-image-label">
                        {t.image}
                        <span>
                          {entry.width} × {entry.height}
                        </span>
                      </div>
                    </div>
                  ) : (
                    <div className="entry-preview">
                      {excerpt(entry.text).slice(0, 240)}
                    </div>
                  )}

                  <div className="entry-meta">
                    <span>{kindLabel(kind, t)}</span>
                    <span className="meta-dot">·</span>
                    {formatTime(entry.created_at)}
                  </div>
                </div>

                {selected && <span className="enter-key">↵</span>}
              </button>

              <button
                type="button"
                className="delete-entry"
                title={t.deleteEntry}
                aria-label={t.deleteEntry}
                onClick={() => onRemove(entry.id)}
              >
                <X size={14} />
              </button>
            </div>
          );
        })
      )}
    </div>
  );
}
