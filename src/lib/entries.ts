import type {
  ClipEntry,
  EntryKind,
  TextEntryKind,
} from '../types';

export function excerpt(text: string): string {
  return text.replace(/\s+/g, ' ').trim();
}

export function classifyTextEntry(text: string): TextEntryKind {
  const normalized = text.trim();

  if (/^https?:\/\//i.test(normalized)) {
    return 'link';
  }

  if (
    /^(const |let |function |import |export |interface |type |class |<\w+|\{[\s\S]*\})/.test(
      normalized
    )
  ) {
    return 'code';
  }

  return 'text';
}

export function getEntryKind(entry: ClipEntry): EntryKind {
  return entry.kind === 'image'
    ? 'image'
    : classifyTextEntry(entry.text);
}

function searchableText(entry: ClipEntry): string {
  if (entry.kind === 'image') {
    return `画像 image スクリーンショット screenshot ${entry.width ?? ''}x${entry.height ?? ''}`;
  }

  return entry.text;
}

export function filterEntries(
  entries: ClipEntry[],
  query: string
): ClipEntry[] {
  const normalized = query.trim().toLocaleLowerCase();

  if (!normalized) {
    return entries;
  }

  return entries.filter((entry) =>
    searchableText(entry).toLocaleLowerCase().includes(normalized)
  );
}
