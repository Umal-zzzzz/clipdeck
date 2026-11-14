# ClipDeck

**macOS専用の、Windowsの `Win + V` に近いクリップボード履歴パレット。**

Rust + Tauri 2 + React/TypeScript によるローカルファーストのMVP実装です。

> **状態:** ソース実装済み。Linux上で作成したため、macOSでのRustビルドおよびアクセシビリティ権限付きの実機テストは未実施です。公開配布する前にMacで動作確認してください。

## 機能

- **Control + Option + V**（`⌃⌥V`）で履歴パレットを表示/非表示
- 350ms間隔でmacOSの `NSPasteboard.changeCount` を監視し、新しいテキストまたはPNG/TIFF画像を履歴化（同じテキストを再コピーした場合も先頭へ移動）
- 最新100件をSQLiteでローカル保存（テキスト最大500KB/件、画像最大20MB/件・画像保存合計250MB）
- 一覧のクリックまたは矢印キー+Enterで選択→元のアプリを前面に戻して `⌘V` を送信
- 検索、個別削除、全件削除、監視の一時停止
- メニューバー常駐、ログイン時自動起動のオン・オフ
- データ送信・アカウント登録不要

## 必要なもの

- **macOS 12以降**（Apple SiliconのMacを想定）
- Xcode Command Line Tools: `xcode-select --install`
- Rust 1.90以降: https://rustup.rs/
- Node.js 20.19以上または22以上 + npm

### 1. 開発モードで起動

```bash
cd clipdeck
npm install
npm run tauri dev
```

起動するとDockではなくMacのメニューバーに常駐します。別アプリでテキストをコピーし、**⌃ + ⌥ + V**で履歴を開いてください。メニューバーのアイコンからも履歴を開けます。

> Rustのインストールを確認: `rustc --version && cargo --version`。`cargo: command not found` の場合はRustツールチェーンのインストールとシェル再起動を行ってください。

### 2. 自動貼り付けの許可

元のアプリへ入力を戻す機能では、macOSのシステムAPIと `/usr/bin/osascript` の固定AppleScriptを使います。

1. `システム設定 → プライバシーとセキュリティ → アクセシビリティ` を開く。
2. **ClipDeck**（開発モードではバイナリ/ターミナルへの許可を求められる場合あり）を許可する。
3. 「ClipDeckがSystem Eventsを制御しようとしています」等の**オートメーション**許可が出たら内容を確認して承認する。
4. 許可後も反映されない場合はClipDeckを終了して再起動する。

自動貼り付けに失敗しても、**選んだテキストはクリップボードへコピー済み**です。エラーがあればパレット内に表示します。対象アプリが終了していたり、保護された入力欄などでは自動貼り付けできない場合があります。

### 3. DMGを作成

```bash
npm run tauri build -- --bundles dmg
```

出力先：`src-tauri/target/release/bundle/dmg/`。

`src-tauri/target/release/bundle/macos/ClipDeck.app` の生成も必要な場合：

```bash
npm run tauri build -- --bundles app,dmg
```

自分用にローカルビルドしたDMGで構いませんが、他者への配布にはApple Developer ID署名とNotarization（公証）を推奨します。

## 操作

| キー・操作 | 動作 |
| --- | --- |
| `⌃⌥V` | パレット表示・非表示 |
| `↑` / `↓` | 履歴選択 |
| `Enter` | 選択項目を貼り付け |
| 項目をクリック | 選択項目を貼り付け |
| `Esc` | パレットを閉じる |
| 検索ボックス | 全履歴を部分一致検索 |
| 「記録中」をクリック | 監視を一時停止・再開 |
| 設定アイコン | ログイン時起動、履歴全削除 |

## データ保存とプライバシー

- `app_data_dir` 配下の `clipdeck.sqlite3` にSQLite形式で保存します（一般的なmacOSでは `~/Library/Application Support/jp.umal.clipdeck/`）。
- 自動履歴は**機密テキストやパスワードも記録する可能性**があります。機密情報のコピーが多い場合は一時停止するか、履歴を削除してください。
- 現時点ではアプリごとの除外、パスワードマネージャーの自動検出、データ暗号化、ファイルの履歴に対応していません。
- プライバシー要件を強化するまでは日常利用のテスト版として扱ってください。
- 認証情報などのコピーを避けたい場合、今後は `[NSPasteboardTypeConcealed]` 等の検知、アプリ除外設定、メモリ限定保存を追加する想定です。

## プロジェクト構成

```text
clipdeck/
├── src/
│   ├── App.tsx                 # 画面レイアウト
│   ├── components/             # 履歴一覧・設定パネル
│   ├── hooks/useClipDeck.ts    # 画面状態と操作
│   ├── services/clipdeck.ts    # Tauri IPCの型付き境界
│   ├── lib/                    # 検索・表示整形などの純粋な処理
│   ├── types.ts                # フロントエンドのデータ型
│   ├── constants.ts            # 共通設定値
│   ├── i18n.ts                 # 日本語/英語のメッセージ
│   ├── assets/clipdeck-icon.png # UIのブランドアイコン
│   ├── main.tsx                # React起動
│   └── styles.css              # ウィンドウUI
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # DB/監視/ホットキー/トレイ/Tauriコマンド
│   │   ├── image_clip.rs       # PNG/TIFFの変換とサムネイル
│   │   ├── mac.rs              # AppKit/AppleScriptのmacOS専用処理
│   │   └── main.rs
│   ├── capabilities/default.json
│   ├── icons/
│   ├── Info.plist              # Dockに出さないLSUIElement
│   ├── tauri.conf.json
│   └── Cargo.toml
├── package.json
└── vite.config.ts
```

## 開発時のチェック

```bash
npm run typecheck
npm run build
npm run check:rust
npm run format:rust:check
```

Rustのコンパイルと実機動作確認はmacOS上で行ってください。LinuxやWindowsでは、macOS専用APIのためアプリを起動できません。

## MVPの制約・今後の改善

1. **macOS実機での検証が未完了です。** Tauri/Apple APIの組み合わせはバージョンや権限状態によって調整が必要になる可能性があります。
2. アプリを戻す処理とキーストローク送信の間に230msの待機があります。アプリによっては、焦点復帰を監視する方式へ改善が必要です。
3. 貼り付け時に選択文字列をプレーンテキストとして書き戻すため、元のコピーのリッチテキスト形式は失われます。
4. パレットは画面中央に表示します。アクティブウィンドウ近くへの配置とマルチモニタ対応は次の段階です。
5. 初期ショートカットは固定です。衝突する場合は `src-tauri/src/lib.rs` の `Code::KeyV` / Modifiers とUIの表示を変更してください。
6. OSのクリップボードアクセス制限・他の常駐アプリとの競合により、監視や自動貼り付けが制限される場合があります。

## 実装で使用している参考文書

- https://v2.tauri.app/plugin/global-shortcut/
- https://v2.tauri.app/plugin/autostart/
- https://v2.tauri.app/learn/system-tray/
- https://v2.tauri.app/distribute/dmg/
- https://docs.rs/objc2-app-kit/latest/objc2_app_kit/struct.NSPasteboard.html

## 画像履歴（v0.2追加）

- PNG/TIFF画像をクリップボードから読み込み、PNGとしてSQLiteの新テーブル `clip_images` に保存。既存 `clips` テーブルは変更しません。
- 画像一覧には180×112px以内のPNGサムネイルを表示し、原寸PNGは貼り付けまでRust側で保持します。
- クリップボードに画像をコピーするには、選択範囲スクショなら **Control + Shift + Command + 4** を使用。通常の Shift + Command + 4 はデフォルトでファイルへ保存され、クリップボード履歴には入りません。
- 画像が大きすぎる場合（クリップボード側64MB超、保存PNG20MB超、3000万画素超）、履歴化しません。画像全体が250MBを超える場合は古い画像から削除します。
- PNG/TIFF以外の画像ペイロードやファイルURL、複数画像の同時コピーは次の段階です。
- PNG画像が貼り付けられるアプリが対象です。標準ペーストボード形式を使っていますが、アプリによって挙動が異なります。
- 新規テーブル追加以外の既存SQLiteデータ変更はありません。念のため更新前に `clipdeck.sqlite3` をバックアップしてください。
