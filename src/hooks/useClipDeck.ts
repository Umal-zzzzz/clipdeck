import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import type { KeyboardEvent } from 'react';

import { DEFAULT_SHORTCUT } from '../constants';
import { defaultLanguage, messages } from '../i18n';
import type { Language } from '../i18n';
import { filterEntries } from '../lib/entries';
import {
  clearHistory,
  deleteEntry,
  getAutostart,
  getLanguage,
  getMonitoring,
  getPrivacySettings,
  getShortcut,
  hidePalette,
  listEntries,
  onHistoryUpdated,
  onPaletteOpened,
  pasteEntry,
  setAutostart,
  setLanguage,
  setMonitoring,
  setPrivacySetting,
  setShortcut,
} from '../services/clipdeck';
import type { ClipEntry, PrivacySettings } from '../types';

const INITIAL_PRIVACY: PrivacySettings = {
  history_ttl_hours: 24,
  clipboard_ttl_minutes: 0,
};

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

export function useClipDeck() {
  const [language, setCurrentLanguage] =
    useState<Language>(defaultLanguage);
  const t = messages[language];

  const [entries, setEntries] = useState<ClipEntry[]>([]);
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [monitoring, setMonitoringState] = useState(true);
  const [autostart, setAutostartState] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [shortcut, setShortcutState] =
    useState(DEFAULT_SHORTCUT);
  const [recordingShortcut, setRecordingShortcut] =
    useState(false);
  const [shortcutBusy, setShortcutBusy] = useState(false);
  const [shortcutError, setShortcutError] = useState('');
  const [privacy, setPrivacy] =
    useState<PrivacySettings>(INITIAL_PRIVACY);

  const searchRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const shortcutRef = useRef<HTMLButtonElement>(null);

  const refresh = useCallback(async () => {
    try {
      setEntries(await listEntries());
    } catch (cause) {
      setError(t.readError(errorMessage(cause)));
    }
  }, [t]);

  const resetPalette = useCallback(() => {
    setQuery('');
    setSelectedIndex(0);
    setError('');
    setSettingsOpen(false);
    setConfirmClear(false);
    setRecordingShortcut(false);

    void refresh();

    requestAnimationFrame(() => {
      searchRef.current?.focus();
    });
  }, [refresh]);

  useEffect(() => {
    void refresh();

    void getMonitoring()
      .then(setMonitoringState)
      .catch((cause) => setError(errorMessage(cause)));

    void getAutostart()
      .then(setAutostartState)
      .catch(() => {
        // Autostart is optional; the settings screen can still be used.
      });

    void getShortcut()
      .then(setShortcutState)
      .catch((cause) => setError(errorMessage(cause)));

    void getPrivacySettings()
      .then(setPrivacy)
      .catch((cause) => setError(errorMessage(cause)));

    void getLanguage()
      .then((saved) => {
        if (saved) {
          setCurrentLanguage(saved);
        }
      })
      .catch((cause) => setError(errorMessage(cause)));

    const listeners = Promise.all([
      onHistoryUpdated(() => {
        void refresh();
      }),
      onPaletteOpened(resetPalette),
    ]);

    return () => {
      void listeners.then((unlisten) =>
        unlisten.forEach((stopListening) => stopListening())
      );
    };
  }, [refresh, resetPalette]);

  const filteredEntries = useMemo(
    () => filterEntries(entries, query),
    [entries, query]
  );

  const activeIndex = Math.min(
    selectedIndex,
    Math.max(0, filteredEntries.length - 1)
  );
  const selectedEntry = filteredEntries[activeIndex];

  useEffect(() => {
    listRef.current
      ?.querySelector<HTMLElement>('[data-selected="true"]')
      ?.scrollIntoView({ block: 'nearest' });
  }, [activeIndex, filteredEntries]);

  const close = useCallback(async () => {
    setError('');
    await hidePalette();
  }, []);

  const paste = useCallback(
    async (id: number) => {
      if (busy) return;

      setBusy(true);
      setError('');

      try {
        await pasteEntry(id);
        await refresh();
      } catch (cause) {
        setError(errorMessage(cause));
      } finally {
        setBusy(false);
      }
    },
    [busy, refresh]
  );

  const remove = useCallback(
    async (id: number) => {
      try {
        await deleteEntry(id);
        setSelectedIndex(0);
        await refresh();
      } catch (cause) {
        setError(errorMessage(cause));
      }
    },
    [refresh]
  );

  const toggleMonitoring = useCallback(async () => {
    try {
      setMonitoringState(
        await setMonitoring(!monitoring)
      );
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }, [monitoring]);

  const toggleAutostart = useCallback(async () => {
    try {
      setAutostartState(await setAutostart(!autostart));
    } catch (cause) {
      setError(t.autostartError(errorMessage(cause)));
    }
  }, [autostart, t]);

  const changeLanguage = useCallback(
    async (next: Language) => {
      try {
        setCurrentLanguage(await setLanguage(next));
        setError('');
      } catch (cause) {
        setError(t.saveError(errorMessage(cause)));
      }
    },
    [t]
  );

  const changePrivacy = useCallback(
    async (key: keyof PrivacySettings, value: number) => {
      try {
        setPrivacy(await setPrivacySetting(key, value));
        setError('');
        await refresh();
      } catch (cause) {
        setError(t.saveError(errorMessage(cause)));
      }
    },
    [refresh, t]
  );

  const clearAll = useCallback(async () => {
    if (!confirmClear) {
      setConfirmClear(true);
      return;
    }

    try {
      await clearHistory();
      setConfirmClear(false);
      setSelectedIndex(0);
      await refresh();
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }, [confirmClear, refresh]);

  const captureShortcut = useCallback(
    (event: KeyboardEvent<HTMLButtonElement>) => {
      if (!recordingShortcut) return;

      event.preventDefault();
      event.stopPropagation();

      if (event.key === 'Escape') {
        setRecordingShortcut(false);
        setShortcutError('');
        return;
      }

      if (
        event.repeat ||
        ['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)
      ) {
        return;
      }

      const modifiers = [
        event.ctrlKey && 'Control',
        event.altKey && 'Alt',
        event.shiftKey && 'Shift',
        event.metaKey && 'Super',
      ].filter(
        (item): item is string => Boolean(item)
      );

      if (modifiers.length < 2) {
        setShortcutError(t.shortcutModsError);
        return;
      }

      const validKey =
        /^(Key[A-Z]|Digit[0-9]|Space|F(?:[1-9]|1[0-2]))$/;

      if (!validKey.test(event.code)) {
        setShortcutError(t.shortcutKeyError);
        return;
      }

      const value = [...modifiers, event.code].join('+');

      setShortcutBusy(true);
      setShortcutError('');

      void setShortcut(value)
        .then((saved) => {
          setShortcutState(saved);
          setRecordingShortcut(false);
        })
        .catch((cause) => {
          setShortcutError(errorMessage(cause));
        })
        .finally(() => {
          setShortcutBusy(false);
        });
    },
    [recordingShortcut, t]
  );

  const startShortcutRecording = useCallback(() => {
    setRecordingShortcut(true);
    setShortcutError('');

    requestAnimationFrame(() => {
      shortcutRef.current?.focus();
    });
  }, []);

  const closeSettings = useCallback(() => {
    setSettingsOpen(false);
    setConfirmClear(false);
    setRecordingShortcut(false);
    setShortcutError('');
    searchRef.current?.focus();
  }, []);

  const toggleSettings = useCallback(() => {
    setSettingsOpen((open) => !open);
    setConfirmClear(false);
  }, []);

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLDivElement>) => {
      if (event.key === 'Escape') {
        event.preventDefault();

        if (settingsOpen) {
          closeSettings();
        } else {
          void close();
        }
      } else if (settingsOpen) {
        return;
      } else if (event.key === 'ArrowDown') {
        event.preventDefault();
        setSelectedIndex((index) =>
          Math.min(index + 1, Math.max(0, filteredEntries.length - 1))
        );
      } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        setSelectedIndex((index) => Math.max(0, index - 1));
      } else if (event.key === 'Enter' && selectedEntry) {
        event.preventDefault();
        void paste(selectedEntry.id);
      }
    },
    [
      close,
      closeSettings,
      filteredEntries.length,
      paste,
      selectedEntry,
      settingsOpen,
    ]
  );

  return {
    language,
    t,
    entries: filteredEntries,
    query,
    selectedIndex: activeIndex,
    monitoring,
    autostart,
    settingsOpen,
    confirmClear,
    busy,
    error,
    shortcut,
    recordingShortcut,
    shortcutBusy,
    shortcutError,
    privacy,
    searchRef,
    listRef,
    shortcutRef,
    onKeyDown,
    onQueryChange: (value: string) => {
      setQuery(value);
      setSelectedIndex(0);
    },
    clearQuery: () => {
      setQuery('');
      setSelectedIndex(0);
    },
    selectIndex: setSelectedIndex,
    cancelShortcutRecording: () => setRecordingShortcut(false),
    close,
    paste,
    remove,
    toggleMonitoring,
    toggleAutostart,
    changeLanguage,
    changePrivacy,
    clearAll,
    captureShortcut,
    startShortcutRecording,
    closeSettings,
    toggleSettings,
  };
}
