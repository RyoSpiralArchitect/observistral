# OBSTRAL

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Providers](https://img.shields.io/badge/providers-Mistral%20%7C%20OpenAI%20%7C%20Anthropic%20%7C%20HF-purple)

> `OBSTRAL` は `observistral`（Mistral 向けログ解析プロトタイプ）から発展した Rust 実装です。
> Python 版 `observistral` はレガシー実装として `src/observistral/` に併存しています。

**複数の LLM プロバイダを同じ操作感で扱える、Rust 製シングルバイナリのチャット実行基盤。**
CLI・REPL・ローカル Web UI をひとつに統合し、モード・ペルソナ・diff レビュー・ストリーミングを共通の UX で提供します。

- **プロバイダ**: OpenAI-compatible / Mistral / Anthropic / HF（Python subprocess）
- **モード**: `実況` / `壁打ち` / `diff批評` / `VIBE` / `ログ解析`
- **ペルソナ**: `default` / `novelist` / `cynical` / `cheerful` / `thoughtful`
- **ストリーミング**: OpenAI・Mistral は SSE ストリーミング対応

---

## クイックスタート

> **デフォルトプロバイダは `mistral`（モデル: `devstral-2`）です。**
> `MISTRAL_API_KEY` を設定すれば `cargo run` だけで起動します。

### Web UI — Mistral（デフォルト）

**bash:**
```bash
export MISTRAL_API_KEY="your-key"
cargo run -- serve --provider mistral
# → http://127.0.0.1:8080 をブラウザで開く
```

**PowerShell:**
```powershell
cd C:\Users\user\observistral
$env:MISTRAL_API_KEY = "your-key"

# 開発用: build + UI起動（cargo を隠蔽）
.\scripts\run-ui.ps1
# → http://127.0.0.1:8080 をブラウザで開く
```

`obstral` コマンドが見つからない場合は、インストール:

```powershell
.\scripts\install.ps1
```

ポートを変えたい場合:

```powershell
.\scripts\run-ui.ps1 -Host 127.0.0.1 -Port 3000
# または（install 後）
obstral serve --host 127.0.0.1 --port 3000
```

### Web UI — OpenAI-compatible で起動する場合

```bash
export OBS_API_KEY="your-openai-key"
cargo run -- serve --provider openai-compatible
# → openai-compatible + gpt-4o-mini
```

### ワンショット

```bash
# VIBE プリセット（Mistral devstral-2 + VIBE モードを自動設定）
export MISTRAL_API_KEY="your-key"
obstral "この設計どう思う？" --vibe

# プロバイダを明示（デフォルトは openai-compatible + gpt-4o-mini）
obstral chat "Hello" --provider openai-compatible --model gpt-4o-mini
```

### REPL（対話セッション）

```bash
obstral repl --provider mistral
# または
obstral --repl
```

---

## API キー設定

**シェル履歴に残るため `--api-key` より環境変数を推奨します。**

| プロバイダ | 環境変数 | 備考 |
|-----------|---------|------|
| Mistral | `MISTRAL_API_KEY` | 必須 |
| Anthropic | `ANTHROPIC_API_KEY` | 必須 |
| OpenAI-compatible | `OBS_API_KEY` または `OPENAI_API_KEY` | ローカルエンドポイントは不要 |
| HF | (不要) | `scripts/hf_infer.py` 経由 |

**bash:**
```bash
export MISTRAL_API_KEY="..."
export ANTHROPIC_API_KEY="..."
export OBS_API_KEY="..."   # OpenAI / vLLM / LM Studio など
```

**PowerShell:**
```powershell
$env:MISTRAL_API_KEY    = "..."
$env:ANTHROPIC_API_KEY  = "..."
$env:OBS_API_KEY        = "..."
```

---

## 例

### ログ解析

```bash
obstral chat "重要ポイントだけ教えて" --mode ログ解析 --log-file ./sample.log --provider mistral --model mistral-large-latest
```

### HF（ローカル推論）

```bash
obstral chat "要約して" --provider hf --model gpt2 --device cpu
# ローカルファイルのみ
obstral chat "要約して" --provider hf --model gpt2 --hf-local-only
```

---

## モード

| モード | エイリアス | 説明 |
|--------|-----------|------|
| `実況` | `jikkyo`, `live` | 操作・思考を逐一ナレーション |
| `壁打ち` | `kabeuchi`, `ideation` | アイデア整理・トレードオフの壁打ち相手 |
| `diff批評` | `diff`, `review` | コードレビュー（バグ・リスク・テスト観点） |
| `VIBE` | `vibe` | 設計〜実装を素早く叩き出す vibe コーディング |

---

## ペルソナ

| ペルソナ | 説明 |
|---------|------|
| `default` | バランス重視・簡潔・実用的 |
| `novelist` | 情景描写・比喩を交えた文体 |
| `cynical` | 鋭い批判・弱い前提を容赦なく指摘 |
| `cheerful` | 明るく前向き・励まし重視 |
| `thoughtful` | 前提確認・段階的・トレードオフ明示 |

---

## CLI リファレンス

```
obstral [PROMPT] [OPTIONS]
obstral chat <PROMPT> [OPTIONS]
obstral repl [OPTIONS]
obstral serve [--host HOST] [--port PORT] [OPTIONS]
obstral list <providers|modes|personas>
```

### 共通オプション

| オプション | デフォルト | 説明 |
|-----------|-----------|------|
| `--vibe` | — | Mistral devstral-2 + VIBE モードのプリセット（下記参照） |
| `--provider` | `mistral` | プロバイダ選択 |
| `--model` | `devstral-2` | モデル名（プロバイダごとに異なる） |
| `--api-key` | — | APIキー（env var 推奨） |
| `--base-url` | — | カスタムエンドポイント |
| `--mode` | `壁打ち` | モード選択 |
| `--persona` | `default` | ペルソナ選択 |
| `--temperature` | `0.7` | サンプリング温度（0〜2） |
| `--max-tokens` | `1024` | 最大トークン数 |
| `--timeout-seconds` | `120` | タイムアウト（秒） |
| `--diff-file <PATH>` | — | diff ファイルを読み込みプロンプトに注入 |
| `--stdin` | — | 標準入力をプロンプトに追加 |

### `--vibe` の優先順位

`--vibe` は **未指定の項目のみ** を補完するプリセットです。

- `--vibe` 単体: `provider=mistral`・`model=devstral-2`・`mode=VIBE` を自動設定
- `--vibe` と `--provider anthropic` を同時指定: **明示した `--provider` が優先**（`--vibe` は `model` と `mode` のみ補完）
- `--vibe` と `--mode diff批評` を同時指定: **明示した `--mode` が優先**

```bash
# すべて vibe 任せ
obstral "認証周りのリファクタ案" --vibe

# provider だけ上書き（mode と model は vibe が補完）
obstral "コードレビューして" --vibe --provider anthropic

# mode だけ上書き（provider と model は vibe が補完）
obstral "この差分見て" --vibe --mode diff批評
```

### 利用例

```bash
# VIBE プリセット（Mistral devstral-2）
export MISTRAL_API_KEY="..."
cargo run -- "認証周りのリファクタ案を出して" --vibe

# Anthropic
export ANTHROPIC_API_KEY="..."
cargo run -- chat "コードレビューして" --provider anthropic --model claude-opus-4-6 --mode diff批評

# ローカル vLLM / LM Studio（APIキー不要）
cargo run -- chat "Hello" --provider openai-compatible --model local-model --base-url http://localhost:8000/v1

# diff ファイルを使ったコードレビュー
cargo run -- "この変更どうかな" --mode diff批評 --diff-file ./changes.diff --persona cynical

# stdin からプロンプト追加
git diff HEAD~1 | cargo run -- "この差分をレビューして" --mode diff批評 --stdin --vibe

# 一覧表示
cargo run -- list providers
cargo run -- list modes
cargo run -- list personas
```

---

## REPL コマンド

REPL 起動後、`/` で始まる行はコマンドとして処理されます。

```
/help                         コマンド一覧
/exit  /quit                  終了
/reset                        会話履歴をクリア
/config                       現在の設定を表示（APIキーは非表示）
/vibe                         VIBE プリセットを適用

/provider <name>              openai-compatible | mistral | anthropic
/model <name>                 モデル名を変更
/base-url <url>               エンドポイントを変更
/mode <name>                  実況 | 壁打ち | diff批評 | VIBE
/persona <name>               default | novelist | cynical | cheerful | thoughtful
/temperature <0..2>
/max-tokens <n>
```

プロンプト表示例:
```
obstral[VIBE|default|mistral]>
```

入力履歴は `.obstral_history` に自動保存されます。

---

## セキュリティ / ローカル保存

- **`.obstral_history`**: REPL の入力履歴のみ保存。会話内容は保存されません。
- **`/config`**: APIキーは `****` で伏せて表示されます。
- **Web UI バインド**: デフォルトは `127.0.0.1`（ローカルホストのみ）。`0.0.0.0` にしない限り外部から到達できません。
- **送信先**: 入力内容は選択したプロバイダの API エンドポイントに送信されます。HF（ローカル推論）は外部送信なし。

---

## Windows セットアップ

```powershell
# PATH に Cargo を追加
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

# ビルド
cargo build

# 実行
cargo run
```

`LNK1181: cannot open input file 'kernel32.lib'` が出る場合は Windows SDK をインストール:

```powershell
winget install Microsoft.WindowsSDK.10.0.18362
```

---

## 環境変数一覧

| 変数 | 説明 |
|------|------|
| `MISTRAL_API_KEY` | Mistral APIキー |
| `ANTHROPIC_API_KEY` | Anthropic APIキー |
| `OBS_API_KEY` | OpenAI-compatible APIキー |
| `OPENAI_API_KEY` | OpenAI APIキー（フォールバック） |
| `OBS_PROVIDER` | デフォルトプロバイダ |
| `OBS_MODEL` | デフォルトモデル |
| `OBS_PERSONA` | デフォルトペルソナ |
| `OBS_BASE_URL` | デフォルトエンドポイント |
| `OBS_TIMEOUT_SECONDS` | タイムアウト秒数 |
| `OBS_HF_DEVICE` | HFデバイス: `auto` / `cpu` / `cuda` |
| `OBS_HF_LOCAL_ONLY` | `1` でオフラインモード（HF） |

---

## Français (FR)

OBSTRAL est un runtime de chatbot avec abstraction de fournisseurs — CLI, REPL et UI web locale dans un seul binaire Rust.
Issu du prototype `observistral` (analyse de logs Mistral), OBSTRAL supporte désormais OpenAI, Anthropic et HF en plus de Mistral.

**Démarrage rapide :**

```bash
# Avec Mistral (recommandé)
export MISTRAL_API_KEY="..."
cargo run -- serve --provider mistral
# → ouvrir http://127.0.0.1:8080
```

```powershell
# PowerShell
$env:MISTRAL_API_KEY = "..."
.\scripts\run-ui.ps1
```

> Par défaut (`cargo run` sans options) : provider `mistral` + modèle `devstral-2`.

**Modes :** `実況` (narration) · `壁打ち` (idéation) · `diff批評` (revue de code) · `VIBE` (vibe coding)

**Personas :** `default` · `novelist` · `cynical` · `cheerful` · `thoughtful`

**Revue de patch :**

```bash
cargo run -- "Critique ce diff" --mode diff批評 --persona cynical --diff-file ./changes.diff --provider anthropic --model claude-opus-4-6
```
