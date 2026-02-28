# OBSTRAL (observistral)

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-local%20web-2dd4bf)

Dual-pane "dual brain" coding cockpit: **Coder** (does the work) + **Observer** (audits it).

- Multi-provider: Mistral, Codestral, OpenAI-compatible, Anthropic, Gemini (OpenAI-compat endpoint), plus optional Mistral CLI.
- Local tools: `mkdir` / `write_file` / `run_command` / `read_file` / `list_files`
- Approvals: edit / command approval with a **Pending edits** queue
- Anti-loop UX: loop detection + warning pill + psychedelic hue shift
- Observer intensity: `polite` / `critical` / `brutal`

![OBSTRAL Web UI](docs/ui.png)

Language: [日本語](#日本語) | [English](#english) | [Français](#français)

---

## 日本語

### これは何？
OBSTRAL は、LLM を「チャット」ではなく**実行基盤**として扱うためのローカルUIです。
Coder がコードやコマンドを提案し、Observer が失敗や危険を監査します。壊れ方も含めてログとして残せます。

特に狙っているのは次の2つです。

- 「モデルが抽象論に逃げて前に進まない」を検出して止める（ループ検出）
- 「ファイル作成やコマンド実行」を、承認付きで確実に発火させる（ツール + 承認）

### クイックスタート（Python Lite 推奨: WDACでEXEが動かない場合）
Python標準ライブラリだけで動く Lite サーバです。Windowsで一番安定します。

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# open http://127.0.0.1:18080/
```

別ワークスペース（例: `C:\Users\user`）配下も触りたい場合:

```powershell
cd C:\Users\user\observistral
python .\scripts\serve_lite.py --host 127.0.0.1 --port 18080 --workspace C:\Users\user
```

### クイックスタート（Rust版）

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui.ps1 -Host 127.0.0.1 -Port 18080
```

WDACで `obstral.exe` がブロックされる場合は、Liteを使ってください（上）。

### 使い方（最重要: Coderにローカル作業をさせる）
1. UIの `Tool root` を設定します（例: `projects/maze-game`）
2. `Edit approval` / `Command approval` をONにします（安全のため）
3. Coder にこう投げます:

```text
迷路ゲームのリポを実際に作って。フォルダ/ファイルを作成して。自分でやって。
```

期待する挙動:

- 返答が `[pending approval]` になり、`Pending edits` に `mkdir` / `write_file` / `run_command` が並ぶ
- 承認すると、実際にフォルダとファイルが作られる

モデルがツール呼び出し非対応でも、OBSTRALは次の「暗黙ツール」を抽出します。

- `path` 行 + コードブロック: ファイルとして `write_file` 扱い
- ` ```bash` / ` ```powershell` / ` ```cmd` のブロック: `run_command` 扱い（WindowsではbashをPowerShellへ保守的に変換）

### 設定（プロバイダ/モデル/キー）
UIは `Chat` / `Code` / `Observer` でプロバイダとキーを分けられます。

環境変数（例）:

```powershell
$env:MISTRAL_API_KEY = "..."
$env:CODESTRAL_API_KEY = "..."
$env:OBS_API_KEY = "..."        # OpenAI互換（OpenAI / vLLM / LM Studio等）
$env:OPENAI_API_KEY = "..."     # OpenAI公式を使う場合はこちらでもOK
$env:ANTHROPIC_API_KEY = "..."
$env:GEMINI_API_KEY = "..."     # または GOOGLE_API_KEY
```

デフォルトBase URL（Lite）:

- Mistral: `https://api.mistral.ai/v1`
- Codestral: `https://codestral.mistral.ai/v1`
- OpenAI-compatible: `https://api.openai.com/v1`
- Anthropic: `https://api.anthropic.com/v1`
- Gemini(OpenAI-compat): `https://generativelanguage.googleapis.com/v1beta/openai`

### ループ検出（Observerが同じレビューを繰り返す対策）
Observerの出力が直近と高類似になったら、UIで警告し、ループ深度に応じて背景が色相シフトします。
さらに、Observer側プロンプトには「差分のみ書け / 繰り返すなら固定文言で止まれ」を注入します。

### ローカル保存について
- スレッドはブラウザの `localStorage` に保存されます（自分のPC内）
- サーバ側はローカルで動きますが、プロバイダへのリクエスト自体は外部APIに送信されます

### セキュリティ注意
- **公開サーバとして使わないでください。** `127.0.0.1` バインド前提です
- `run_command` は強力です。デモでは承認ON推奨

---

## English

### What is this?
OBSTRAL is a **local coding cockpit**: **Coder** proposes and executes changes, **Observer** audits failures/risks.
It is designed to make "agentic coding" reliable: tools, approvals, and loop-breaking.

### Quick Start (Python Lite, recommended on Windows/WDAC)

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# open http://127.0.0.1:18080/
```

To allow working under another workspace root (example: `C:\Users\user`):

```powershell
python .\scripts\serve_lite.py --host 127.0.0.1 --port 18080 --workspace C:\Users\user
```

### Quick Start (Rust)

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui.ps1 -Host 127.0.0.1 -Port 18080
```

If WDAC blocks `obstral.exe`, use Lite.

### Getting the Coder to actually create files/folders
1. Set `Tool root` (example: `projects/maze-game`)
2. Keep `Edit approval` / `Command approval` ON
3. Tell the Coder:

```text
Create the maze game repo for real. Create folders/files locally. Do it yourself.
```

Expected:
- Coder replies with `[pending approval]`
- You approve items in **Pending edits**
- Files/folders are created locally

Even if a model does not support tool calling, OBSTRAL extracts "implied tools" from common output patterns:
- A standalone file path line + a fenced code block => `write_file`
- ` ```bash` / ` ```powershell` / ` ```cmd` blocks => `run_command` (bash is conservatively translated to PowerShell on Windows)

### Providers / Keys
You can configure `Chat` / `Code` / `Observer` providers separately.

Environment variables:
- `MISTRAL_API_KEY`, `CODESTRAL_API_KEY`
- `OBS_API_KEY` (OpenAI-compatible), or `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `GEMINI_API_KEY` (or `GOOGLE_API_KEY`)

Default Lite base URLs:
- Mistral: `https://api.mistral.ai/v1`
- Codestral: `https://codestral.mistral.ai/v1`
- OpenAI-compatible: `https://api.openai.com/v1`
- Anthropic: `https://api.anthropic.com/v1`
- Gemini (OpenAI-compat): `https://generativelanguage.googleapis.com/v1beta/openai`

### Loop detection / Observer intensity
If Observer repeats itself, UI shows a loop warning and applies a psychedelic hue shift.
Observer can be set to `polite` / `critical` / `brutal`.

### Security notes
This is meant to be **local-only**. Do not expose it publicly.
`run_command` is powerful; keep approvals enabled.

---

## Français

### C'est quoi ?
OBSTRAL est un **cockpit de dev local**: **Coder** exécute, **Observer** audite.
Le but est de rendre l'agentic coding concret: outils, approvals, anti-boucle.

### Démarrage rapide (Python Lite, recommandé sur Windows/WDAC)

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# ouvrir http://127.0.0.1:18080/
```

Changer la racine de workspace (ex: `C:\Users\user`):

```powershell
python .\scripts\serve_lite.py --host 127.0.0.1 --port 18080 --workspace C:\Users\user
```

### Démarrage rapide (Rust)

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui.ps1 -Host 127.0.0.1 -Port 18080
```

Si WDAC bloque `obstral.exe`, utilisez Lite.

### Forcer le Coder à créer vraiment des fichiers/dossiers
1. Réglez `Tool root` (ex: `projects/maze-game`)
2. Laissez `Edit approval` / `Command approval` activés
3. Demandez au Coder:

```text
Crée le repo du jeu de labyrinthe, pour de vrai. Crée les dossiers/fichiers localement. Fais-le toi-même.
```

Même si le modèle ne supporte pas les tool calls, OBSTRAL extrait des "outils implicites":
- ligne de chemin + bloc de code => `write_file`
- blocs `bash/powershell/cmd` => `run_command` (bash traduit de façon conservative vers PowerShell sur Windows)

### Providers / clés
Configuration séparée pour `Chat` / `Code` / `Observer`.

Variables d'environnement:
- `MISTRAL_API_KEY`, `CODESTRAL_API_KEY`
- `OBS_API_KEY` (OpenAI-compatible) ou `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `GEMINI_API_KEY` (ou `GOOGLE_API_KEY`)

URLs par défaut (Lite):
- Mistral: `https://api.mistral.ai/v1`
- Codestral: `https://codestral.mistral.ai/v1`
- OpenAI-compatible: `https://api.openai.com/v1`
- Anthropic: `https://api.anthropic.com/v1`
- Gemini (OpenAI-compat): `https://generativelanguage.googleapis.com/v1beta/openai`

### Sécurité
Pensé pour une utilisation locale (`127.0.0.1`). Ne pas exposer publiquement.
`run_command` est puissant; gardez les approvals.

---

## License
MIT

