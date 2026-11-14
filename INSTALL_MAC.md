# Macで最初にやること

Terminalで、ZIPを展開したフォルダに移動してください。

```bash
cd ~/Downloads/clipdeck  # 展開場所に合わせて変更
xcode-select --install   # 既に入っていれば不要
rustc --version          # 1.90以上が必要
cargo --version
node --version           # Node 22推奨
npm install
npm run tauri dev
```

メニューバーにアイコンが表示されたら、テキストをコピーして `Control + Option + V` を押します。

元のアプリに自動貼り付けできない場合は、システム設定のアクセシビリティとオートメーション権限を確認してください。

DMG作成：

```bash
npm run tauri build -- --bundles dmg
```

生成結果は `src-tauri/target/release/bundle/dmg/` です。未署名のローカルDMGなので、一般配布時には署名・公証が必要です。

**重要:** このプロジェクトはmacOS実機でのコンパイル・動作検証前のMVPソースです。エラーが出た場合はそのログを基に調整してください。
