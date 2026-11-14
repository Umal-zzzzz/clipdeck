import type { Language, Messages } from '../i18n';

const SHORTCUT_SYMBOLS: Record<string, string> = {
  Control: '⌃',
  Ctrl: '⌃',
  Alt: '⌥',
  Option: '⌥',
  Shift: '⇧',
  Super: '⌘',
  Meta: '⌘',
  Command: '⌘',
  Cmd: '⌘',
};

export function formatShortcut(value: string): string {
  return value
    .split('+')
    .map(
      (part) =>
        SHORTCUT_SYMBOLS[part] ??
        part.replace(/^Key/, '').replace(/^Digit/, '')
    )
    .join(' ');
}

export function formatEntryTime(
  timestampMs: number,
  language: Language,
  t: Messages,
  nowMs = Date.now()
): string {
  const delta = Math.max(0, nowMs - timestampMs);

  if (delta < 60_000) {
    return t.now;
  }

  if (delta < 3_600_000) {
    const minutes = Math.floor(delta / 60_000);
    return t.minutesAgo(minutes);
  }

  if (delta < 86_400_000) {
    const hours = Math.floor(delta / 3_600_000);
    return t.hoursAgo(hours);
  }

  return new Intl.DateTimeFormat(
    language === 'ja' ? 'ja-JP' : 'en-US',
    { month: 'short', day: 'numeric' }
  ).format(timestampMs);
}
