import type { KeyboardEvent, RefObject } from 'react';
import { Trash2, X } from 'lucide-react';

import {
  CLIPBOARD_OPTIONS,
  HISTORY_OPTIONS,
} from '../constants';
import type { Language, Messages } from '../i18n';
import type { PrivacySettings } from '../types';

interface SettingsPanelProps {
  language: Language;
  autostart: boolean;
  privacy: PrivacySettings;
  confirmClear: boolean;
  recordingShortcut: boolean;
  shortcutBusy: boolean;
  shortcutError: string;
  shortcut: string;
  shortcutRef: RefObject<HTMLButtonElement | null>;
  t: Messages;
  onClose: () => void;
  onLanguageChange: (language: Language) => void;
  onShortcutStart: () => void;
  onShortcutCapture: (
    event: KeyboardEvent<HTMLButtonElement>
  ) => void;
  onShortcutBlur: () => void;
  onPrivacyChange: (
    key: keyof PrivacySettings,
    value: number
  ) => void;
  onToggleAutostart: () => void;
  onClearAll: () => void;
}

export function SettingsPanel({
  language,
  autostart,
  privacy,
  confirmClear,
  recordingShortcut,
  shortcutBusy,
  shortcutError,
  shortcut,
  shortcutRef,
  t,
  onClose,
  onLanguageChange,
  onShortcutStart,
  onShortcutCapture,
  onShortcutBlur,
  onPrivacyChange,
  onToggleAutostart,
  onClearAll,
}: SettingsPanelProps) {
  const historyLabels = [
    t.off,
    t.oneHour,
    t.sixHours,
    t.oneDay,
    t.oneWeek,
    t.thirtyDays,
  ];
  const clipboardLabels = [
    t.off,
    t.oneMinute,
    t.fiveMinutes,
    t.fifteenMinutes,
  ];

  return (
    <section className="settings-panel">
      <div className="settings-panel-header">
        <div className="settings-title">{t.settings}</div>

        <button
          type="button"
          className="icon-btn"
          aria-label={t.close}
          title={t.close}
          onClick={onClose}
        >
          <X size={16} aria-hidden="true" />
        </button>
      </div>

      <label className="setting-select-label" htmlFor="language">
        {t.language}
      </label>

      <select
        id="language"
        className="setting-select"
        value={language}
        onChange={(event) => {
          const next = event.target.value;
          if (next === 'ja' || next === 'en') {
            onLanguageChange(next);
          }
        }}
      >
        <option value="en">English</option>
        <option value="ja">日本語</option>
      </select>

      <div className="shortcut-setting">
        <div className="setting-select-label">{t.shortcut}</div>

        <button
          type="button"
          ref={shortcutRef}
          className={`shortcut-recorder ${
            recordingShortcut ? 'recording' : ''
          }`}
          disabled={shortcutBusy}
          onClick={onShortcutStart}
          onKeyDown={onShortcutCapture}
          onBlur={onShortcutBlur}
        >
          {shortcutBusy
            ? t.shortcutSaving
            : recordingShortcut
              ? t.shortcutRecord
              : shortcut}
        </button>

        <div className="shortcut-hint">{t.shortcutHint}</div>

        {shortcutError && (
          <div className="shortcut-error" role="alert">
            {shortcutError}
          </div>
        )}
      </div>

      <label
        className="setting-select-label"
        htmlFor="historyRetention"
      >
        {t.historyRetention}
      </label>

      <select
        id="historyRetention"
        className="setting-select"
        value={privacy.history_ttl_hours}
        onChange={(event) =>
          onPrivacyChange(
            'history_ttl_hours',
            Number(event.target.value)
          )
        }
      >
        {HISTORY_OPTIONS.map((hours, index) => (
          <option key={hours} value={hours}>
            {historyLabels[index]}
          </option>
        ))}
      </select>

      <div className="privacy-hint">{t.historyNote}</div>

      <label
        className="setting-select-label"
        htmlFor="clipboardExpiry"
      >
        {t.clipboardExpiry}
      </label>

      <select
        id="clipboardExpiry"
        className="setting-select"
        value={privacy.clipboard_ttl_minutes}
        onChange={(event) =>
          onPrivacyChange(
            'clipboard_ttl_minutes',
            Number(event.target.value)
          )
        }
      >
        {CLIPBOARD_OPTIONS.map((minutes, index) => (
          <option key={minutes} value={minutes}>
            {clipboardLabels[index]}
          </option>
        ))}
      </select>

      <div className="privacy-hint">{t.clipboardNote}</div>

      <button
        type="button"
        className="setting-row"
        onClick={onToggleAutostart}
      >
        <span>{t.startup}</span>

        <span className={`switch ${autostart ? 'on' : ''}`}>
          <span />
        </span>
      </button>

      <div className="privacy-hint">{t.privacyNote}</div>

      <button
        type="button"
        className={`clear-button ${confirmClear ? 'armed' : ''}`}
        onClick={onClearAll}
      >
        <Trash2 size={14} />
        {confirmClear ? t.confirmClear : t.clearAll}
      </button>
    </section>
  );
}
