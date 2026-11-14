#[cfg(not(target_os = "macos"))]
compile_error!("ClipDeck is designed for macOS. Build it on a Mac.");

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "macos")]
mod image_clip;

#[cfg(target_os = "macos")]
mod application {
    use super::{image_clip, mac};
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use rusqlite::{params, Connection, OptionalExtension};
    use serde::Serialize;
    use std::{
        fs,
        sync::{
            atomic::{AtomicBool, AtomicI64, Ordering},
            Arc, Mutex,
        },
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use tauri::{
        menu::{Menu, MenuItem},
        tray::TrayIconBuilder,
        AppHandle, Emitter, Manager, WindowEvent,
    };
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    const MAX_ENTRIES: usize = 100;
    const MAX_TEXT_BYTES: usize = 500_000;
    const POLL_MS: u64 = 350;
    const MAX_CLIPBOARD_IMAGE_BYTES: usize = 64 * 1024 * 1024;
    const MAX_IMAGE_STORAGE_BYTES: i64 = 250 * 1024 * 1024;

    pub struct AppState {
        db: Mutex<Connection>,
        monitoring: Arc<AtomicBool>,
        last_target: Mutex<Option<i32>>,
        shortcut: Mutex<Shortcut>,
        clipboard_ttl_minutes: Arc<AtomicI64>,
    }
    #[derive(Serialize)]
    struct PrivacySettings {
        history_ttl_hours: i64,
        clipboard_ttl_minutes: i64,
    }

    #[derive(Serialize)]
    struct ClipEntry {
        id: i64,
        text: String,
        created_at: i64,
        kind: String,
        thumbnail: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    }

    fn now_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    fn open_database(app: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
        let directory = app.path().app_data_dir()?;
        fs::create_dir_all(&directory)?;
        let connection = Connection::open(directory.join("clipdeck.sqlite3"))?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
            PRAGMA secure_delete = ON;
             CREATE TABLE IF NOT EXISTS clips (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS clips_created_idx ON clips(created_at DESC, id DESC);
             CREATE TABLE IF NOT EXISTS clip_images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                fingerprint TEXT NOT NULL UNIQUE,
                png BLOB NOT NULL,
                thumbnail BLOB NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                created_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS clip_images_created_idx ON clip_images(created_at DESC, id DESC);
             CREATE TABLE IF NOT EXISTS app_settings (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );"
        )?;
        Ok(connection)
    }
    const HISTORY_TTLS: &[i64] = &[0, 1, 6, 24, 168, 720];
    const CLIPBOARD_TTLS: &[i64] = &[0, 1, 5, 15];

    fn read_numeric_setting(db: &Connection, key: &str, fallback: i64, allowed: &[i64]) -> i64 {
        db.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| allowed.contains(value))
        .unwrap_or(fallback)
    }

    fn write_setting(db: &Connection, key: &str, value: &str) -> Result<(), String> {
        db.execute(
            "INSERT INTO app_settings (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key)
            DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn read_privacy(db: &Connection) -> PrivacySettings {
        PrivacySettings {
            history_ttl_hours: read_numeric_setting(db, "history_ttl_hours", 24, HISTORY_TTLS),
            clipboard_ttl_minutes: read_numeric_setting(
                db,
                "clipboard_ttl_minutes",
                0,
                CLIPBOARD_TTLS,
            ),
        }
    }

    fn delete_expired_history(db: &Connection) -> Result<usize, String> {
        let hours = read_privacy(db).history_ttl_hours;

        if hours == 0 {
            return Ok(0);
        }

        let cutoff = now_ms().saturating_sub(hours.saturating_mul(3_600_000));

        let text_count = db
            .execute("DELETE FROM clips WHERE created_at <= ?1", [cutoff])
            .map_err(|e| e.to_string())?;

        let image_count = db
            .execute("DELETE FROM clip_images WHERE created_at <= ?1", [cutoff])
            .map_err(|e| e.to_string())?;

        let total = text_count + image_count;

        if total > 0 {
            let _ = db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }

        Ok(total)
    }

    fn cleanup_history(app: &AppHandle) -> Result<(), String> {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;

        let deleted = delete_expired_history(&db)?;
        drop(db);

        if deleted > 0 {
            let _ = app.emit("history-updated", ());
        }

        Ok(())
    }

    #[tauri::command]
    fn get_privacy_settings(state: tauri::State<'_, AppState>) -> Result<PrivacySettings, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        Ok(read_privacy(&db))
    }

    #[tauri::command]
    fn set_privacy_setting(
        app: AppHandle,
        key: String,
        value: i64,
    ) -> Result<PrivacySettings, String> {
        let allowed = match key.as_str() {
            "history_ttl_hours" => HISTORY_TTLS,
            "clipboard_ttl_minutes" => CLIPBOARD_TTLS,
            _ => return Err("Unknown privacy setting".into()),
        };

        if !allowed.contains(&value) {
            return Err("Unsupported retention period".into());
        }

        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;

        write_setting(&db, &key, &value.to_string())?;

        if key == "clipboard_ttl_minutes" {
            state.clipboard_ttl_minutes.store(value, Ordering::Relaxed);
        }

        let deleted = if key == "history_ttl_hours" {
            delete_expired_history(&db)?
        } else {
            0
        };

        let settings = read_privacy(&db);
        drop(db);

        if deleted > 0 {
            let _ = app.emit("history-updated", ());
        }

        Ok(settings)
    }

    #[tauri::command]
    fn get_language(state: tauri::State<'_, AppState>) -> Result<String, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;

        Ok(db
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'language'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_default())
    }

    #[tauri::command]
    fn set_language(app: AppHandle, language: String) -> Result<String, String> {
        if language != "ja" && language != "en" {
            return Err("Unsupported language".into());
        }

        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;

        write_setting(&db, "language", &language)?;

        Ok(language)
    }

    // Keep newest 100 entries across both tables, without migrating old text rows.
    fn prune_history(tx: &rusqlite::Transaction<'_>) -> Result<(), String> {
        let old_ids = {
            let mut query = tx
                .prepare(
                    "SELECT id FROM (
                   SELECT id, created_at FROM clips
                   UNION ALL
                   SELECT -id AS id, created_at FROM clip_images
                 ) ORDER BY created_at DESC, id DESC LIMIT -1 OFFSET ?1",
                )
                .map_err(|e| e.to_string())?;
            let rows = query
                .query_map([MAX_ENTRIES as i64], |row| row.get::<_, i64>(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };
        for id in old_ids {
            if id > 0 {
                tx.execute("DELETE FROM clips WHERE id = ?1", [id])
                    .map_err(|e| e.to_string())?;
            } else {
                tx.execute("DELETE FROM clip_images WHERE id = ?1", [-id])
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    // Images have a separate disk budget; oldest images are discarded first.
    fn prune_image_storage(tx: &rusqlite::Transaction<'_>) -> Result<(), String> {
        let old_ids = {
            let mut query = tx
                .prepare(
                    "SELECT id, length(png) + length(thumbnail) FROM clip_images
                 ORDER BY created_at DESC, id DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = query
                .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
                .map_err(|e| e.to_string())?;
            let mut total = 0i64;
            let mut old_ids = Vec::new();
            for row in rows {
                let (id, bytes) = row.map_err(|e| e.to_string())?;
                if total.saturating_add(bytes) > MAX_IMAGE_STORAGE_BYTES {
                    old_ids.push(id);
                } else {
                    total += bytes;
                }
            }
            old_ids
        };
        for id in old_ids {
            tx.execute("DELETE FROM clip_images WHERE id = ?1", [id])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn store_clip(app: &AppHandle, text: &str) -> Result<(), String> {
        if text.trim().is_empty() || text.len() > MAX_TEXT_BYTES {
            return Ok(());
        }
        let state = app.state::<AppState>();
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        let tx = db.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO clips (text, created_at) VALUES (?1, ?2)
             ON CONFLICT(text) DO UPDATE SET created_at=excluded.created_at",
            params![text, now_ms()],
        )
        .map_err(|e| e.to_string())?;
        prune_history(&tx)?;
        tx.commit().map_err(|e| e.to_string())?;
        drop(db);
        let _ = app.emit("history-updated", ());
        Ok(())
    }

    fn store_image(app: &AppHandle, image: image_clip::StoredImage) -> Result<(), String> {
        let state = app.state::<AppState>();
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        let tx = db.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO clip_images (fingerprint, png, thumbnail, width, height, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(fingerprint) DO UPDATE SET created_at=excluded.created_at",
            params![
                image.fingerprint,
                image.png,
                image.thumbnail,
                image.width,
                image.height,
                now_ms()
            ],
        )
        .map_err(|e| e.to_string())?;
        prune_history(&tx)?;
        prune_image_storage(&tx)?;
        tx.commit().map_err(|e| e.to_string())?;
        drop(db);
        let _ = app.emit("history-updated", ());
        Ok(())
    }

    #[tauri::command]
    fn list_entries(state: tauri::State<'_, AppState>) -> Result<Vec<ClipEntry>, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut query = db
            .prepare(
                "SELECT id, kind, text, thumbnail, width, height, created_at FROM (
                SELECT id, 'text' AS kind, text, NULL AS thumbnail,
                       NULL AS width, NULL AS height, created_at FROM clips
                UNION ALL
                SELECT -id AS id, 'image' AS kind, '' AS text, thumbnail,
                       width, height, created_at FROM clip_images
             ) ORDER BY created_at DESC, id DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = query
            .query_map([MAX_ENTRIES as i64], |row| {
                let thumbnail: Option<Vec<u8>> = row.get(3)?;
                Ok(ClipEntry {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    text: row.get(2)?,
                    thumbnail: thumbnail
                        .map(|bytes| format!("data:image/png;base64,{}", BASE64.encode(bytes))),
                    width: row.get(4)?,
                    height: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    fn delete_entry(
        app: AppHandle,
        state: tauri::State<'_, AppState>,
        id: i64,
    ) -> Result<(), String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if id > 0 {
            db.execute("DELETE FROM clips WHERE id = ?1", [id])
                .map_err(|e| e.to_string())?;
        } else if id < 0 {
            db.execute("DELETE FROM clip_images WHERE id = ?1", [-id])
                .map_err(|e| e.to_string())?;
        }
        let _ = app.emit("history-updated", ());
        Ok(())
    }

    #[tauri::command]
    fn clear_history(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
        state
            .db
            .lock()
            .map_err(|e| e.to_string())?
            .execute_batch("DELETE FROM clips; DELETE FROM clip_images;")
            .map_err(|e| e.to_string())?;
        let _ = app.emit("history-updated", ());
        Ok(())
    }

    fn parse_shortcut(value: &str) -> Result<Shortcut, String> {
        if value.len() > 80 {
            return Err("Too long shortcut keys".into());
        }

        let shortcut: Shortcut = value
            .parse()
            .map_err(|_| "Invalid shortcut format".to_string())?;

        let count = [
            Modifiers::CONTROL,
            Modifiers::ALT,
            Modifiers::SHIFT,
            Modifiers::META,
        ]
        .iter()
        .filter(|flag| shortcut.mods.contains(**flag))
        .count();

        if count < 2 {
            return Err("Use at least two modifier keys".into());
        }

        let key = value.rsplit('+').next().unwrap_or_default();

        let allowed = (key.len() == 4
            && key.starts_with("Key")
            && key.as_bytes()[3].is_ascii_uppercase())
            || (key.len() == 6 && key.starts_with("Digit") && key.as_bytes()[5].is_ascii_digit())
            || key == "Space"
            || matches!(
                key,
                "F1" | "F2"
                    | "F3"
                    | "F4"
                    | "F5"
                    | "F6"
                    | "F7"
                    | "F8"
                    | "F9"
                    | "F10"
                    | "F11"
                    | "F12"
            );

        if !allowed {
            return Err("Only use Alphabet, Numbers, Space key, Function keys".into());
        }

        Ok(shortcut)
    }

    #[tauri::command]
    fn get_shortcut(state: tauri::State<'_, AppState>) -> Result<String, String> {
        let current = state.shortcut.lock().map_err(|e| e.to_string())?;

        Ok(current.into_string())
    }

    #[tauri::command]
    fn set_shortcut(app: AppHandle, value: String) -> Result<String, String> {
        let next = parse_shortcut(&value)?;
        let state = app.state::<AppState>();

        let mut current = state.shortcut.lock().map_err(|e| e.to_string())?;

        if *current == next {
            return Ok(current.into_string());
        }

        // Register first so a failed update leaves the old shortcut active.
        app.global_shortcut().register(next).map_err(|e| {
            format!("Unable to register. It may be conflicting with other apps.: {e}")
        })?;

        if let Err(e) = app.global_shortcut().unregister(*current) {
            let _ = app.global_shortcut().unregister(next);

            return Err(format!("Unable to remove the previous shortcut: {e}"));
        }

        let saved = state.db.lock().map_err(|e| e.to_string()).and_then(|db| {
            db.execute(
                "INSERT INTO app_settings (key, value)
                    VALUES ('shortcut', ?1)
                    ON CONFLICT(key)
                    DO UPDATE SET value = excluded.value",
                [next.into_string()],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        });

        // Restore the old registration if persistence fails.
        if let Err(e) = saved {
            let _ = app.global_shortcut().register(*current);
            let _ = app.global_shortcut().unregister(next);

            return Err(format!("Unable to save the shortcut: {e}"));
        }

        *current = next;

        Ok(next.into_string())
    }
    #[tauri::command]
    fn get_monitoring(state: tauri::State<'_, AppState>) -> bool {
        state.monitoring.load(Ordering::Relaxed)
    }

    #[tauri::command]
    fn set_monitoring(state: tauri::State<'_, AppState>, enabled: bool) -> bool {
        state.monitoring.store(enabled, Ordering::Relaxed);
        enabled
    }

    #[tauri::command]
    fn hide_palette(app: AppHandle) -> Result<(), String> {
        app.get_webview_window("main")
            .ok_or("Window not found")?
            .hide()
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    async fn paste_entry(app: AppHandle, id: i64) -> Result<(), String> {
        enum Content {
            Text(String),
            Image(Vec<u8>),
        }
        let content = {
            let state = app.state::<AppState>();
            let db = state.db.lock().map_err(|e| e.to_string())?;
            if id > 0 {
                Content::Text(
                    db.query_row("SELECT text FROM clips WHERE id = ?1", [id], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()
                    .map_err(|e| e.to_string())?
                    .ok_or("History entry not found")?,
                )
            } else if id < 0 {
                Content::Image(
                    db.query_row("SELECT png FROM clip_images WHERE id = ?1", [-id], |row| {
                        row.get::<_, Vec<u8>>(0)
                    })
                    .optional()
                    .map_err(|e| e.to_string())?
                    .ok_or("Image entry not found")?,
                )
            } else {
                return Err("Invalid history ID".into());
            }
        };

        // Content is restored before focus-and-paste, so manual Cmd+V still works on error.
        match &content {
            Content::Text(text) => mac::write_text(text)?,
            Content::Image(png) => mac::write_image(png)?,
        }
        {
            let state = app.state::<AppState>();
            let db = state.db.lock().map_err(|e| e.to_string())?;
            if id > 0 {
                db.execute(
                    "UPDATE clips SET created_at = ?1 WHERE id = ?2",
                    params![now_ms(), id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                db.execute(
                    "UPDATE clip_images SET created_at = ?1 WHERE id = ?2",
                    params![now_ms(), -id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        let _ = app.emit("history-updated", ());
        let target_pid = *app
            .state::<AppState>()
            .last_target
            .lock()
            .map_err(|e| e.to_string())?;
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        window.hide().map_err(|e| e.to_string())?;

        if let Some(pid) = target_pid.filter(|pid| *pid != std::process::id() as i32) {
            let result = tauri::async_runtime::spawn_blocking(move || mac::focus_and_paste(pid))
                .await
                .map_err(|e| e.to_string())?;
            if let Err(error) = result {
                let _ = window.show();
                let _ = window.set_focus();
                return Err(error);
            }
        }
        Ok(())
    }

    fn start_clipboard_watcher(
        app: AppHandle,
        monitoring: Arc<AtomicBool>,
        clipboard_ttl: Arc<AtomicI64>,
    ) {
        thread::spawn(move || {
            let mut previous_count = mac::clipboard_change_count();
            let mut previous_ttl = clipboard_ttl.load(Ordering::Relaxed);
            let mut pending_clear: Option<(isize, Instant)> = None;
            let mut last_cleanup = Instant::now();

            loop {
                thread::sleep(Duration::from_millis(POLL_MS));

                if last_cleanup.elapsed() >= Duration::from_secs(30) {
                    let _ = cleanup_history(&app);
                    last_cleanup = Instant::now();
                }

                let current_ttl = clipboard_ttl.load(Ordering::Relaxed);
                if current_ttl != previous_ttl {
                    pending_clear = None;
                    previous_ttl = current_ttl;
                }

                let next_count = mac::clipboard_change_count();
                if next_count != previous_count {
                    previous_count = next_count;
                    pending_clear = if current_ttl > 0 {
                        let delay = Duration::from_secs((current_ttl * 60) as u64);
                        Some((next_count, Instant::now() + delay))
                    } else {
                        None
                    };

                    if monitoring.load(Ordering::Relaxed) {
                        if let Some(image) = mac::read_image(MAX_CLIPBOARD_IMAGE_BYTES) {
                            if let Ok(image) = image_clip::prepare(&image.bytes, image.format) {
                                let _ = store_image(&app, image);
                            }
                        } else if let Some(text) = mac::read_text() {
                            let _ = store_clip(&app, &text);
                        }
                    }
                }

                if let Some((expected_count, deadline)) = pending_clear {
                    if Instant::now() >= deadline {
                        if current_ttl > 0 && mac::clear_clipboard_if_unchanged(expected_count) {
                            previous_count = mac::clipboard_change_count();
                        }
                        pending_clear = None;
                    }
                }
            }
        });
    }

    fn show_palette(app: &AppHandle) {
        let Some(window) = app.get_webview_window("main") else {
            return;
        };
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }
        if let Some(pid) = mac::frontmost_pid().filter(|pid| *pid != std::process::id() as i32) {
            if let Ok(mut last_target) = app.state::<AppState>().last_target.lock() {
                *last_target = Some(pid);
            }
        }
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("palette-opened", ());
    }

    pub fn run() {
        tauri::Builder::default()
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ))
            .setup(|app| {
                app.handle()
                    .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
                let handle = app.handle().clone();
                let db = open_database(&handle)?;
                let saved_shortcut: Option<String> = db
                    .query_row(
                        "SELECT value FROM app_settings WHERE key = 'shortcut'",
                        [],
                        |row| row.get(0),
                    )
                    .optional()?;

                let fallback = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV);

                let initial = saved_shortcut
                    .as_deref()
                    .and_then(|value| parse_shortcut(value).ok())
                    .unwrap_or(fallback);

                let privacy = read_privacy(&db);

                delete_expired_history(&db).map_err(std::io::Error::other)?;

                let clipboard_ttl = Arc::new(AtomicI64::new(privacy.clipboard_ttl_minutes));

                let monitoring = Arc::new(AtomicBool::new(true));

                app.manage(AppState {
                    db: Mutex::new(db),
                    monitoring: monitoring.clone(),
                    last_target: Mutex::new(None),
                    shortcut: Mutex::new(initial),
                    clipboard_ttl_minutes: clipboard_ttl.clone(),
                });

                start_clipboard_watcher(handle.clone(), monitoring, clipboard_ttl);

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, pressed, event| {
                            if event.state() != ShortcutState::Pressed {
                                return;
                            }

                            let state = app.state::<AppState>();

                            let is_current = state
                                .shortcut
                                .lock()
                                .map(|active| *pressed == *active)
                                .unwrap_or(false);

                            if is_current {
                                show_palette(app);
                            }
                        })
                        .build(),
                )?;

                if let Err(error) = app.global_shortcut().register(initial) {
                    if initial == fallback {
                        return Err(Box::new(error));
                    }

                    app.global_shortcut().register(fallback)?;

                    let state = app.state::<AppState>();
                    *state.shortcut.lock().unwrap() = fallback;
                }

                let open_item = MenuItem::with_id(app, "open", "Open history", true, None::<&str>)?;
                let quit_item =
                    MenuItem::with_id(app, "quit", "Quit ClipDeck", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&open_item, &quit_item])?;
                let tray = TrayIconBuilder::new()
                    .icon(tauri::include_image!("icons/tray.png"))
                    .icon_as_template(false)
                    .tooltip("ClipDeck clipboard history")
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "open" => show_palette(app),
                        "quit" => app.exit(0),
                        _ => (),
                    })
                    .build(app)?;
                // App-scoped tray remains alive until the application terminates.
                let _ = tray;
                Ok(())
            })
            .on_window_event(|window, event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                WindowEvent::Focused(false) => {
                    let _ = window.hide();
                }
                _ => (),
            })
            .invoke_handler(tauri::generate_handler![
                list_entries,
                delete_entry,
                clear_history,
                get_monitoring,
                set_monitoring,
                hide_palette,
                paste_entry,
                get_shortcut,
                set_shortcut,
                get_privacy_settings,
                set_privacy_setting,
                get_language,
                set_language,
            ])
            .run(tauri::generate_context!())
            .expect("Unable to start ClipDeck");
    }
}

#[cfg(target_os = "macos")]
pub use application::run;
