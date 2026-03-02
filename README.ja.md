# OBSTRAL

「デュアルブレイン」型のコーディング・コックピット:

- **Coder**: ファイル作成/編集、コマンド実行まで行う（承認ゲートあり）
- **Observer**: 実況しながら批評し、次の一手を提案する（スコア付き）
- **Chat**: 壁打ち/雑談/アイデア出し（実装ループを壊さない）

Languages: [English](README.md) | [Japanese](README.ja.md) | [French](README.fr.md)

## これは何？

多くのLLMツールは「会話」を最適化します。

OBSTRALは「**制御された実行ループ**」を最適化します。

- Coder vs Observer の緊張関係
- proposals のスコアリング + フェーズ制御（core/feature/polish）
- ループ検出（同じ批評、同じ失敗コマンドの反復）
- 安全装置（Edit/Command approval、tool_rootでの隔離）

## 起動（Rustサーバ）

### Web UI

```powershell
.\scripts\run-ui.ps1
```

ブラウザで開く:

- `http://127.0.0.1:18080/`

### TUI

```powershell
.\scripts\run-tui.ps1
```

補足: UI/TUIが共存しやすいよう、スクリプト側で `CARGO_TARGET_DIR` を分離しています。

## Liteサーバ（Python）

RustのEXEが実行できない環境（例: WDACで新規バイナリがブロックされる）向けに、Pythonフォールバックがあります:

```powershell
python .\scripts\serve_lite.py
```

これは互換・救済モードであり、全機能の代替ではありません。

## 重要な概念

### tool_root

エージェントの実行（ファイル/コマンド）は `tool_root` 配下に閉じ込めます。

デフォルトは `.tmp/<thread-id>` で、スレッドごとに隔離して「ネストしたgitリポジトリ事故」を避けます。

### 承認（Approvals）

- **Edit approval**: `write_file` は保留（pending edits）に積まれ、承認後に反映されます
- **Command approval**: `exec` も同様にゲート可能です（任意）

## Providerと実戦エラー

OBSTRALは OpenAI互換APIを中心に、`ChatProvider` traitで複数プロバイダを差し替えられる設計です。

よくある実戦エラー:

- `401 Unauthorized`: APIキー不正/未設定
- `429 Too Many Requests`: レート制限（バックオフが必要）
- `max_tokens` / `max_completion_tokens`: モデルごとのパラメータ差

## セキュリティ

前提はローカル（`127.0.0.1`）運用です。

ネットワークに公開するなら、認証とツール実行の更なるハードニングが必須です。

## トラブルシュート

### GitHubへのpushが `127.0.0.1` 経由で失敗する

環境変数で死んだプロキシが強制されている可能性があります。

PowerShellセッション内で解除:

```powershell
Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY,Env:GIT_HTTP_PROXY,Env:GIT_HTTPS_PROXY -ErrorAction SilentlyContinue
```

### 対話プロンプト無しでpushする（WDAC回避）

環境によっては、gitの対話プロンプトが壊れます（例: `sh.exe` が Win32 error 5 で落ちる）。

GitHubトークンが使えるなら、1回だけ非対話でpushできます:

```powershell
$env:GITHUB_TOKEN = "ghp_..."
.\scripts\push.ps1
```

### `cargo run` が `obstral.exe` を消せず失敗する（アクセス拒否）

同じターゲットのEXEが実行中です。

- `.\scripts\kill-obstral.ps1`
- もしくは `.\scripts\run-ui.ps1` / `.\scripts\run-tui.ps1` を使ってください

## License

MIT
