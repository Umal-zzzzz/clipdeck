use objc2::rc::autoreleasepool;
use objc2_app_kit::{
    NSApplicationActivationOptions, NSPasteboard, NSPasteboardTypePNG, NSPasteboardTypeString,
    NSPasteboardTypeTIFF, NSRunningApplication, NSWorkspace,
};
use objc2_foundation::{NSArray, NSData, NSString};
use std::{process::Command, thread, time::Duration};

pub fn clipboard_change_count() -> isize {
    autoreleasepool(|_| NSPasteboard::generalPasteboard().changeCount())
}

pub fn read_text() -> Option<String> {
    autoreleasepool(|_| {
        let pasteboard = NSPasteboard::generalPasteboard();
        // SAFETY: NSPasteboardTypeString is an AppKit-defined, immutable global constant.
        pasteboard
            .stringForType(unsafe { NSPasteboardTypeString })
            .map(|value| value.to_string())
    })
}

pub fn write_text(text: &str) -> Result<(), String> {
    autoreleasepool(|_| {
        let pasteboard = NSPasteboard::generalPasteboard();
        let _ = pasteboard.clearContents();
        // SAFETY: NSPasteboardTypeString is an AppKit-defined, immutable global constant.
        if pasteboard
            .setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString })
        {
            Ok(())
        } else {
            Err("macOSのクリップボードに書き込めませんでした".into())
        }
    })
}

pub fn frontmost_pid() -> Option<i32> {
    autoreleasepool(|_| {
        NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .map(|application| application.processIdentifier())
            .filter(|pid| *pid > 0)
    })
}

pub fn focus_and_paste(pid: i32) -> Result<(), String> {
    let activated = autoreleasepool(|_| {
        NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
            .map(|target| target.activateWithOptions(NSApplicationActivationOptions::empty()))
            .unwrap_or(false)
    });

    if !activated {
        return Err("元のアプリを前面に戻せませんでした。クリップボードにはコピー済みです".into());
    }

    // Activation is asynchronous. Let the previous editor regain keyboard focus.
    thread::sleep(Duration::from_millis(230));

    // System Events is a fixed script, not assembled from clipboard data.
    // macOS requires user-granted Accessibility and possibly Automation permission.
    let result = Command::new("/usr/bin/osascript")
        .args([
            "-e",
            "tell application \"System Events\" to keystroke \"v\" using command down",
        ])
        .output()
        .map_err(|e| format!("貼り付けを実行できません: {e}"))?;

    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "自動貼り付けに失敗しました。システム設定 → プライバシーとセキュリティ → アクセシビリティでClipDeckを許可してください。クリップボードへのコピーは完了しています。詳細: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ))
    }
}
pub struct ClipboardImage {
    pub bytes: Vec<u8>,
    pub format: &'static str,
}

pub fn read_image(max_bytes: usize) -> Option<ClipboardImage> {
    autoreleasepool(|_| {
        let pasteboard = NSPasteboard::generalPasteboard();

        for (paste_type, format) in [
            (unsafe { NSPasteboardTypePNG }, "png"),
            (unsafe { NSPasteboardTypeTIFF }, "tiff"),
        ] {
            if let Some(data) = pasteboard.dataForType(paste_type) {
                if !data.is_empty() && data.len() <= max_bytes {
                    return Some(ClipboardImage {
                        bytes: data.to_vec(),
                        format,
                    });
                }
            }
        }

        None
    })
}

pub fn write_image(png: &[u8]) -> Result<(), String> {
    autoreleasepool(|_| {
        let pasteboard = NSPasteboard::generalPasteboard();

        let png_type = unsafe { NSPasteboardTypePNG };
        let types = NSArray::from_slice(&[png_type]);

        unsafe {
            pasteboard.declareTypes_owner(&types, None);
        }

        let data = NSData::with_bytes(png);

        if pasteboard.setData_forType(Some(&data), png_type) {
            Ok(())
        } else {
            Err("画像をクリップボードへ書き込めませんでした".into())
        }
    })
}

pub fn clear_clipboard_if_unchanged(expected_count: isize) -> bool {
    autoreleasepool(|_| {
        let board = NSPasteboard::generalPasteboard();

        if board.changeCount() != expected_count {
            return false;
        }

        let _ = board.clearContents();

        true
    })
}
