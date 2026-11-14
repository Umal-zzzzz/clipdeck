export type ClipEntryType = 'text' | 'image';

export type TextEntryKind = 'link' | 'code' | 'text';

export type EntryKind = ClipEntryType | TextEntryKind;

export interface ClipEntry {
  id: number;
  text: string;
  created_at: number;
  kind: ClipEntryType;
  thumbnail: string | null;
  width: number | null;
  height: number | null;
}

export interface PrivacySettings {
  history_ttl_hours: number;
  clipboard_ttl_minutes: number;
}
