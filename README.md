# OBSTRAL

プロバイダ抽象化付きチャット実行基盤。**Rust 製シングルバイナリ**で CLI・REPL・ローカル Web UI をすべて提供します。

- **プロバイダ**: OpenAI-compatible / Mistral / Anthropic
- **モード**: `実況` / `壁打ち` / `diff批評` / `VIBE`
- **ペルソナ**: `default` / `novelist` / `cynical` / `cheerful` / `thoughtful`
- **ストリーミング**: OpenAI・Mistral は SSE ストリーミング対応

---

## クイックスタート

### Web UI（デフォルト）

```powershell
cd C:\Users\user\observistral
$env:MISTRAL_API_KEY = "your-key"

cargo run
# → http://127.0.0.1:8080 をブラウザで開く
```

ポートを変えたい場合:

```powershell
cargo run -- serve --host 127.0.0.1 --port 3000
```

### ワンショット

```powershell
# 短縮形
cargo run -- "この設計どう思う？" --vibe

# 明示形
cargo run -- chat "Hello" --provider openai-compatible --model gpt-4o-mini
```

### REPL（対話セッション）

```powershell
cargo run -- repl
# または
cargo run -- --repl
```

---

## API キー設定

**シェル履歴に残るため `--api-key` より環境変数を推奨します。**

| プロバイダ | 環境変数 | 備考 |
|-----------|---------|------|
| Mistral | `MISTRAL_API_KEY` | 必須 |
| Anthropic | `ANTHROPIC_API_KEY` | 必須 |
| OpenAI-compatible | `OBS_API_KEY` または `OPENAI_API_KEY` | ローカルエンドポイントは不要 |

```powershell
$env:MISTRAL_API_KEY    = "..."
$env:ANTHROPIC_API_KEY  = "..."
$env:OBS_API_KEY        = "..."   # OpenAI / vLLM / LM Studio など
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
| `--vibe` | — | Mistral devstral-2 + VIBE モードのプリセット |
| `--provider` | `openai-compatible` | プロバイダ選択 |
| `--model` | `gpt-4o-mini` | モデル名 |
| `--api-key` | — | APIキー（env var 推奨） |
| `--base-url` | — | カスタムエンドポイント |
| `--mode` | `壁打ち` | モード選択 |
| `--persona` | `default` | ペルソナ選択 |
| `--temperature` | `0.7` | サンプリング温度（0〜2） |
| `--max-tokens` | `1024` | 最大トークン数 |
| `--timeout-seconds` | `120` | タイムアウト（秒） |
| `--diff-file <PATH>` | — | diff ファイルを読み込みプロンプトに注入 |
| `--stdin` | — | 標準入力をプロンプトに追加 |

### 利用例

```powershell
# VIBE プリセット（Mistral devstral-2）
$env:MISTRAL_API_KEY = "..."
cargo run -- "認証周りのリファクタ案を出して" --vibe

# Anthropic
$env:ANTHROPIC_API_KEY = "..."
cargo run -- chat "コードレビューして" --provider anthropic --model claude-opus-4-6 --mode diff批評

# ローカル vLLM / LM Studio
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

## Python 版（レガシー）

Python 実装 (`observistral`) は Hugging Face ローカル推論対応など一部機能を含みます。

```bash
pip install -e .
pip install -e .[hf]   # HuggingFace ローカル推論

observistral "プロンプト" --provider mistral --model devstral-2
observistral --repl
```

HF ローカル:

```bash
OBS_HF_LOCAL_ONLY=1 observistral "要約して" --provider hf --model mistralai/Mistral-7B-Instruct-v0.2
```

---

## 環境変数一覧

| 変数 | 説明 |
|------|------|
| `MISTRAL_API_KEY` | Mistral APIキー |
| `ANTHROPIC_API_KEY` | Anthropic APIキー |
| `OBS_API_KEY` | OpenAI-compatible APIキー |
| `OPENAI_API_KEY` | OpenAI APIキー（フォールバック） |
| `OBS_PROVIDER` | デフォルトプロバイダ（Python版） |
| `OBS_MODEL` | デフォルトモデル（Python版） |
| `OBS_PERSONA` | デフォルトペルソナ（Python版） |
| `OBS_BASE_URL` | デフォルトエンドポイント（Python版） |
| `OBS_TIMEOUT_SECONDS` | タイムアウト秒数（Python版） |
| `OBS_HF_DEVICE` | HFデバイス: `auto` / `cpu` / `cuda` |
| `OBS_HF_LOCAL_ONLY` | `1` でオフラインモード（HF） |

---

## Français (FR)

OBSTRAL est un runtime de chatbot avec abstraction de fournisseurs — CLI, REPL et UI web locale dans un seul binaire Rust.

**Démarrage rapide :**

```powershell
$env:MISTRAL_API_KEY = "..."
cargo run
# → ouvrir http://127.0.0.1:8080
```

**Modes :** `実況` (narration) · `壁打ち` (idéation) · `diff批評` (revue de code) · `VIBE` (vibe coding)

**Personas :** `default` · `novelist` · `cynical` · `cheerful` · `thoughtful`

**Revue de patch :**

```powershell
cargo run -- "Critique ce diff" --mode diff批評 --persona cynical --diff-file ./changes.diff --provider anthropic --model claude-opus-4-6
```
