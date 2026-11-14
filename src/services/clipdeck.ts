import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  disable,
  enable,
  isEnabled,
} from '@tauri-apps/plugin-autostart';

import type { Language } from '../i18n';
import type {
  ClipEntry,
  PrivacySettings,
} from '../types';

export function listEntries(): Promise<ClipEntry[]> {
  return invoke<ClipEntry[]>('list_entries');
}

export function getMonitoring(): Promise<boolean> {
  return invoke<boolean>('get_monitoring');
}

export function setMonitoring(enabled: boolean): Promise<boolean> {
  return invoke<boolean>('set_monitoring', { enabled });
}

export function getAutostart(): Promise<boolean> {
  return isEnabled();
}

export function setAutostart(enabled: boolean): Promise<boolean> {
  return (enabled ? enable() : disable()).then(isEnabled);
}

export function getShortcut(): Promise<string> {
  return invoke<string>('get_shortcut');
}

export function setShortcut(value: string): Promise<string> {
  return invoke<string>('set_shortcut', { value });
}

export function getPrivacySettings(): Promise<PrivacySettings> {
  return invoke<PrivacySettings>('get_privacy_settings');
}

export function setPrivacySetting(
  key: keyof PrivacySettings,
  value: number
): Promise<PrivacySettings> {
  return invoke<PrivacySettings>('set_privacy_setting', {
    key,
    value,
  });
}

export function getLanguage(): Promise<Language | null> {
  return invoke<string>('get_language').then((value) =>
    value === 'ja' || value === 'en' ? value : null
  );
}

export function setLanguage(language: Language): Promise<Language> {
  return invoke<Language>('set_language', { language });
}

export function hidePalette(): Promise<void> {
  return invoke('hide_palette');
}

export function pasteEntry(id: number): Promise<void> {
  return invoke('paste_entry', { id });
}

export function deleteEntry(id: number): Promise<void> {
  return invoke('delete_entry', { id });
}

export function clearHistory(): Promise<void> {
  return invoke('clear_history');
}

export function onHistoryUpdated(callback: () => void) {
  return listen('history-updated', callback);
}

export function onPaletteOpened(callback: () => void) {
  return listen('palette-opened', callback);
}
