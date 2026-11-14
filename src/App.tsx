import {
  Check,
  Clock3,
  Pause,
  Play,
  Search,
  Settings2,
  X,
} from 'lucide-react';

import { HistoryList } from './components/HistoryList';
import { SettingsPanel } from './components/SettingsPanel';
import { useClipDeck } from './hooks/useClipDeck';
import { formatEntryTime, formatShortcut } from './lib/format';

const appIcon = new URL(
  './assets/clipdeck-icon.png',
  import.meta.url
).href;

export default function App() {
  const deck = useClipDeck();

  return (
    <div
      className="app-shell"
      onKeyDown={deck.onKeyDown}
      lang={deck.language}
    >
      <header className="topbar">
        <div className="brand-mark">
          <img className="brand-icon" src={appIcon} alt="" />
        </div>

        <div className="brand-text">
          <div className="brand-name">ClipDeck</div>
          <div className="brand-subtitle">{deck.t.subtitle}</div>
        </div>

        <span className="shortcut" title={deck.t.shortcut}>
          {formatShortcut(deck.shortcut)}
        </span>

        <button
          type="button"
          className="icon-btn close"
          aria-label={deck.t.close}
          title={`${deck.t.close} (Esc)`}
          onClick={() => void deck.close()}
        >
          <X size={18} />
        </button>
      </header>

      <div className="search-box">
        <Search size={18} className="search-icon" />

        <input
          ref={deck.searchRef}
          value={deck.query}
          onChange={(event) =>
            deck.onQueryChange(event.target.value)
          }
          placeholder={deck.t.search}
          spellCheck={false}
          aria-label={deck.t.search}
        />

        {deck.query && (
          <button
            type="button"
            className="icon-btn"
            aria-label={deck.t.clearSearch}
            onClick={deck.clearQuery}
          >
            <X size={15} />
          </button>
        )}
      </div>

      <div className="section-label">
        <span>
          <Clock3 size={13} />
          {deck.t.recent}
        </span>
        <span>{deck.t.items(deck.entries.length)}</span>
      </div>

      <HistoryList
        entries={deck.entries}
        selectedIndex={deck.selectedIndex}
        query={deck.query}
        monitoring={deck.monitoring}
        busy={deck.busy}
        t={deck.t}
        listRef={deck.listRef}
        formatTime={(timestampMs) =>
          formatEntryTime(timestampMs, deck.language, deck.t)
        }
        onSelect={deck.selectIndex}
        onPaste={(id) => void deck.paste(id)}
        onRemove={(id) => void deck.remove(id)}
      />

      {deck.error && (
        <div className="error-banner" role="alert">
          {deck.error}
        </div>
      )}

      {deck.settingsOpen && (
        <SettingsPanel
          language={deck.language}
          autostart={deck.autostart}
          privacy={deck.privacy}
          confirmClear={deck.confirmClear}
          recordingShortcut={deck.recordingShortcut}
          shortcutBusy={deck.shortcutBusy}
          shortcutError={deck.shortcutError}
          shortcut={formatShortcut(deck.shortcut)}
          shortcutRef={deck.shortcutRef}
          t={deck.t}
          onClose={deck.closeSettings}
          onLanguageChange={(language) =>
            void deck.changeLanguage(language)
          }
          onShortcutStart={deck.startShortcutRecording}
          onShortcutCapture={deck.captureShortcut}
          onShortcutBlur={deck.cancelShortcutRecording}
          onPrivacyChange={(key, value) =>
            void deck.changePrivacy(key, value)
          }
          onToggleAutostart={() => void deck.toggleAutostart()}
          onClearAll={() => void deck.clearAll()}
        />
      )}

      <footer className="bottom-bar">
        <button
          type="button"
          className={`monitor-pill ${
            deck.monitoring ? 'running' : 'stopped'
          }`}
          onClick={() => void deck.toggleMonitoring()}
          title={
            deck.monitoring
              ? deck.t.pauseMonitoring
              : deck.t.resumeMonitoring
          }
        >
          {deck.monitoring ? (
            <Pause size={12} fill="currentColor" />
          ) : (
            <Play size={12} fill="currentColor" />
          )}
          {deck.monitoring ? deck.t.monitoring : deck.t.stopped}
        </button>

        <div className="footer-guide">
          <span>
            <kbd>↑</kbd>
            <kbd>↓</kbd>
            {deck.t.select}
          </span>
          <span>
            <kbd>↵</kbd>
            {deck.t.paste}
          </span>
        </div>

        <button
          type="button"
          className={`icon-btn settings-btn ${
            deck.settingsOpen ? 'active' : ''
          }`}
          aria-label={deck.t.settings}
          title={deck.t.settings}
          onClick={deck.toggleSettings}
        >
          {deck.settingsOpen ? (
            <Check size={16} />
          ) : (
            <Settings2 size={17} />
          )}
        </button>
      </footer>
    </div>
  );
}
