# OBSTRAL (observistral)

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-web%20%2B%20TUI-2dd4bf)

**実装・批評・会話**を分離した、ローカルで動く AI 開発アシスタント。

コーディング LM とチャットボットを同じペインに混在させない。**Coder**（実装）・**Observer**（批評）・**Chat**（会話）の三役を独立させることで、それぞれの品質を高め、開発体験を向上させます。

インターフェースは **2種類**:

| インターフェース | 起動 | 特徴 |
|---|---|---|
| **Web GUI** | `cargo run -- serve` | ブラウザ。設定パネル・diff D&D・マルチスレッド |
| **TUI** | `cargo run -- tui` | ターミナル完結。ratatui デュアルペイン・エージェントループ内蔵 |

Language: [日本語](#日本語) | [English](#english) | [Français](#français)

---

## 日本語

### これは何？

OBSTRAL は、**コーディング LM とチャットボットを役割ごとに分離**した、ローカルで動く AI 開発アシスタントです。

多くの LLM ツールでは「コードを書く・批評する・質問に答える」がすべて同じチャット画面に流れます。OBSTRAL はこれを三役に分けます:

| ペイン | 役割 | 何をするか |
|---|---|---|
| **Coder** | 実装 | コードを書き、コマンドを実行し、ファイルを作る |
| **Observer** | 批評 | Coder の出力を読み、リスク・改善点を独立した視点で指摘する |
| **Chat** | 会話 | 設計相談・仕様確認・質疑応答など、実装から切り離した対話 |

それぞれに **別のモデル・別のプロバイダ** を割り当てられます。たとえば「Coder は Codestral、Observer は GPT-4o、Chat は Claude」という使い方も可能です。

---

### 1. どちらを使うか（30秒で決まる）

| 環境 | 推奨 |
|---|---|
| **Windows**（特に会社PC） | **Python Lite** — WDACブロック回避、Python標準ライブラリのみ |
| **Linux / Mac + ブラウザ派** | **Web GUI** (`cargo run -- serve`) |
| **ターミナル派 / SSH環境** | **TUI** (`cargo run -- tui`) |

---

### 2. クイックスタート

#### Python Lite（Windows推奨）

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# ブラウザで http://127.0.0.1:18080/ を開く
# Lite のデフォルト workspace は `~/obstral-work` です
```

別ディレクトリも操作したい場合:

```powershell
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080 -WorkspaceRoot C:\Users\user
```

##### Lite TUI（Python / ターミナル完結）

```powershell
cd C:\Users\user\observistral
.\scripts\obstral-lite.ps1 tui --lang ja
# or:
python .\scripts\obstral_lite_cli.py tui --lang ja
```

#### Web GUI（Linux / Mac / 管理者権限あり）

```bash
cargo run -- serve
# http://127.0.0.1:8080/ を開く
```

#### TUI（ターミナル完結、SSH可）

```bash
cargo run -- tui
# または:
cargo run -- tui --tool-root projects/maze --auto-observe
cargo run -- tui --model gpt-4o --observer-model gpt-4o-mini
```

WDACで `obstral.exe` がブロックされる場合:
```powershell
Add-MpPreference -ExclusionPath "C:\Users\user\observistral\target"
# それでもブロックされる → Lite を使う
```

---

### 3. APIキーを設定する

UIの設定パネルに直接入力、または環境変数で渡す:

```powershell
$env:OPENAI_API_KEY    = "sk-..."      # OpenAI
$env:ANTHROPIC_API_KEY = "sk-ant-..."  # Claude
$env:MISTRAL_API_KEY   = "..."         # Mistral / Codestral
$env:OBS_API_KEY       = "..."         # OpenAI互換 (vLLM / LM Studio等)
$env:GEMINI_API_KEY    = "..."         # Gemini
```

`Chat` / `Code` / `Observer` それぞれで別プロバイダ・別モデルを指定できます。

---

### 4. TUI — ターミナル UI

SSH や VSCode ターミナルから `cargo run -- tui` で起動するデュアルペイン UI。

```
┌─ OBSTRAL ─────────────────────────────────────────────────────────┐
│ C:gpt-4o-mini  O:gpt-4o-mini  Tab=切替  Ctrl+A=自動  Ctrl+K=停止 │
├─────────────────────────────┬──────────────────────────────────────┤
│ ◉ CODER ⠸ [iter 2/12]       │ ○ OBSERVER                          │
│                             │                                      │
│ you › maze game を作って    │ obs › エラー処理が抜けている         │
│ coder ›                     │                                      │
│ <think>                     │ ── proposals ──                      │
│ goal: ディレクトリ構造作成  │ score: 82 / severity: crit           │
│ risk: パス衝突              │ to_coder: Add input validation        │
│ next: New-Item -Force       │                                      │
│ </think>                    │                                      │
│ [TOOL] New-Item -ItemType…  │                                      │
│ [RESULT] exit=0             │                                      │
├─────────────────────────────┴──────────────────────────────────────┤
│ › CODER  Enter=送信  Shift+Enter=改行  End=最下部                  │
│ > _                                                                │
└────────────────────────────────────────────────────────────────────┘
```

#### TUI キーバインド

| キー | アクション |
|---|---|
| `Tab` | Coder ↔ Observer フォーカス切り替え |
| `Enter` | メッセージ送信 |
| `Shift+Enter` | 改行 |
| `Ctrl+K` | ストリーミング停止 |
| `Ctrl+A` | 自動実況 ON/OFF（Coder 完了時に Observer を自動起動） |
| `Ctrl+O` | Observer を手動トリガー |
| `Ctrl+L` | 現在ペインのメッセージクリア |
| `PageUp / PageDown` | スクロール（5行） |
| `Home / End` | 先頭 / 最下部へジャンプ |
| `Ctrl+C / Esc` | 終了 |

#### TUI オプション

```bash
--model <MODEL>            # Coder モデル（デフォルト: 設定ファイル）
--observer-model <MODEL>   # Observer モデル（省略時は Coder と同じ）
--tool-root <DIR>          # exec コマンドの作業ディレクトリ
--auto-observe             # 起動時から自動実況 ON
```

#### TUI レンダリング

| 表示 | 意味 |
|---|---|
| `⠋⠙⠹⠸⠼⠴⠦⠧` スピナー | ストリーミング中（200ms/フレーム） |
| `[iter N/12]` | エージェントのツール呼び出し回数 |
| `[↑N]` | N行上にスクロール中 |
| `<think>` ブロック | 薄灰色 italic（モデルの推論スクラッチパッド） |
| `[TOOL] コマンド` | 黄色太字（実行されたコマンド） |
| `[RESULT] exit=0` | 緑（成功） |
| `[RESULT] exit=N ⚠` | 赤太字（失敗、モデルが診断必須） |
| `diff --git …` | 青太字（diff ヘッダ） |
| `+` / `-` 行 | 緑 / 赤（diff 追加 / 削除） |
| `@@` 行 | シアン（diff ハンクヘッダ） |

---

### 5. Coderにローカル作業をさせる

1. 設定 → **Tool root** に作業ディレクトリを入力（例: `projects/maze-game`）
2. **Edit approval** / **Command approval** を ON（デフォルトON）
3. Coder にこう送る:

```text
迷路ゲームのリポを実際に作って。フォルダ・ファイルを作成して。自分でやって。
```

**何が起きるか:**

```
Coder が mkdir, git init, ファイル作成を順番に実行
  ↓
Web GUI: bash コードブロックに ▶ run ボタン（クリックで実行）
TUI:     exec ツールが自動で実行 → [TOOL]/[RESULT] がリアルタイム表示
Lite:    tool call を抽出 → 承認待ち（Approveで実行）
  ↓
結果がその場に表示
```

#### Coder の推論強化

TUI の Coder はすべてのツール呼び出し前に自動でスクラッチパッドを出力します:

```
<think>
goal: ディレクトリ構造を作成する
risk: 既存ファイルの上書き
next: New-Item -ItemType Directory -Force -Path src
</think>
```

これにより「間違った方向への突進」（300+ トークンの修正コスト）を ~30 トークンで防止します。

また:
- **出力トランケーション**: stdout 1500文字 / stderr 600文字で自動カット
- **コンテキスト剪定**: 古いツール結果を1行要約に折りたたみ
- **エラー増幅**: 失敗時は診断プロンプトを自動注入
- **最大12イテレーション**（無限ループ防止）

---

### 6. Web GUI の見どころ

#### diff / patch コードブロックの色付け

チャット内に ` ```diff ` ブロックが来ると自動で色付け:

| 行 | 色 |
|---|---|
| `diff …` / `index …` | 青（ファイルヘッダ） |
| `+++ ` / `--- ` | 白太字（パスライン） |
| `@@ ` | シアン（ハンクヘッダ） |
| `+` で始まる行 | 緑（追加） |
| `-` で始まる行 | 赤（削除） |
| コンテキスト行 | 薄白 |

#### その他の UI 機能

- **メッセージタイムスタンプ**: 各メッセージに相対時刻（"2m ago" など）
- **`<think>` ブロック**: モデルの推論スクラッチパッドを薄灰色 italic で表示（本文と区別）
- **提案の展開/折りたたみ**: `toCoder` 詳細は `▶ details` クリックで表示
- **ループ検出**: Observer が同じ批評を繰り返すと警告 pill + 画面が色相シフト

#### diff 批評モード

設定パネルの **diff** エリアにファイルをドラッグ&ドロップするか直接貼り付けると、Observer に差分をレビューさせられます。

---

### 7. Observer を活用する

Observer は Coder の出力を**独立した視点**で読み、批評と改善提案を出します。Coder のコンテキストを汚染しないため、指摘が的確になります。

**強度の選び方:**

| 強度 | 使いどき |
|---|---|
| `丁寧` | アイデア段階、壊したくない |
| `批評` | 通常開発、バランス重視 |
| `容赦なし` | リリース前、アーキテクチャレビュー |

**Observer が出す提案の読み方:**

- **score (0–100)**: 優先度。80以上は今すぐやる、30以下は後回し可
- **phase**: `core`（基盤未安定）/ `feature`（機能追加中）/ `polish`（仕上げ段階）
- **cost**: `low` / `medium` / `high` — 実装コスト
- **impact**: 何が改善・修正されるか一行で

現在フェーズ外の提案はカードが暗転し、スコア順に自動ソートされます。

**ループ検出:**
Observer が同じ批評を繰り返し始めると、UI が警告 pill を表示 + 画面が色相シフトします。
これが出たら Observer の入力欄に新しい観点を追加するか、Coder の成果物を更新してください。

---

### 8. セキュリティ注意

- **`127.0.0.1` バインド専用**。公開サーバとして使わないでください
- `run_command` / `▶ run` / TUI exec は強力です。デモ中は承認 ON を推奨
- スレッドはブラウザの `localStorage` に保存されます（サーバ側に残りません）

---

## English

### What is this?

OBSTRAL is a **local AI development assistant that separates implementation, critique, and conversation** into distinct roles.

Most LLM tools put "write code", "review code", and "answer questions" in the same chat window. OBSTRAL splits these into three dedicated roles:

| Pane | Role | What it does |
|---|---|---|
| **Coder** | Implementation | Writes code, runs commands, creates files |
| **Observer** | Critique | Reads Coder's output and flags risks, issues, and improvements — independently |
| **Chat** | Conversation | Design discussions, questions, spec clarification — separate from implementation context |

Each role can use a **different model and provider**. For example: Coder on Codestral, Observer on GPT-4o, Chat on Claude.

**Two interfaces:**

| Interface | Command | Best for |
|---|---|---|
| **Web GUI** | `cargo run -- serve` | Browser, settings panel, diff drag & drop, multi-thread |
| **TUI** | `cargo run -- tui` | Terminal-only, SSH, ratatui dual-pane, built-in agentic loop |

---

### 1. Which version to use?

| Environment | Recommendation |
|---|---|
| **Windows** (especially corporate) | **Python Lite** — safe mode for locked-down Windows (WDAC), pure stdlib |
| **Linux / Mac + browser** | **Web GUI** (`cargo run -- serve`) |
| **Terminal / SSH** | **TUI** (`cargo run -- tui`) |

---

### 2. Quick Start

#### Python Lite (recommended on Windows)

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# open http://127.0.0.1:18080/
# Lite default workspace: `~/obstral-work`
```

To extend the workspace root:
```powershell
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080 -WorkspaceRoot C:\Users\user
```

##### Lite TUI (Python / terminal-only)

```powershell
cd C:\Users\user\observistral
.\scripts\obstral-lite.ps1 tui --lang en
# or:
python .\scripts\obstral_lite_cli.py tui --lang en
```

#### Web GUI

```bash
cargo run -- serve
# open http://127.0.0.1:8080/
```

#### TUI

```bash
cargo run -- tui
# or with options:
cargo run -- tui --tool-root projects/maze --auto-observe
cargo run -- tui --model gpt-4o --observer-model gpt-4o-mini
```

If WDAC blocks `obstral.exe`:
```powershell
Add-MpPreference -ExclusionPath "C:\Users\user\observistral\target"
# Still blocked? Use Lite instead.
```

---

### 3. API Keys

Set in the UI settings panel, or via environment variables:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export MISTRAL_API_KEY="..."
export OBS_API_KEY="..."      # OpenAI-compatible (vLLM, LM Studio, etc.)
export GEMINI_API_KEY="..."
```

`Chat`, `Code`, and `Observer` can each use different providers and models.

---

### 4. TUI — Terminal UI

Launch with `cargo run -- tui`. A full ratatui dual-pane terminal UI — no browser needed.

```
┌─ OBSTRAL ─────────────────────────────────────────────────────────┐
│ C:gpt-4o-mini  O:gpt-4o-mini  Tab=switch  Ctrl+A=auto  Ctrl+K=stop│
├─────────────────────────────┬──────────────────────────────────────┤
│ ◉ CODER ⠸ [iter 2/12]       │ ○ OBSERVER                          │
│                             │                                      │
│ you › build maze game       │ obs › error handling missing         │
│ coder ›                     │                                      │
│ <think>                     │ ── proposals ──                      │
│ goal: create dir structure  │ score: 82 / severity: crit           │
│ risk: path collision        │ to_coder: Add input validation        │
│ next: New-Item -Force       │                                      │
│ </think>                    │                                      │
│ [TOOL] New-Item -ItemType…  │                                      │
│ [RESULT] exit=0             │                                      │
├─────────────────────────────┴──────────────────────────────────────┤
│ › CODER  Enter=send  Shift+Enter=newline  End=bottom               │
│ > _                                                                │
└────────────────────────────────────────────────────────────────────┘
```

#### TUI Key Bindings

| Key | Action |
|---|---|
| `Tab` | Switch focus Coder ↔ Observer |
| `Enter` | Send message |
| `Shift+Enter` | Insert newline |
| `Ctrl+K` | Stop streaming |
| `Ctrl+A` | Toggle auto-observe (fires Observer on each Coder response) |
| `Ctrl+O` | Trigger Observer manually |
| `Ctrl+L` | Clear current pane |
| `PageUp / PageDown` | Scroll 5 lines |
| `Home / End` | Jump to top / bottom |
| `Ctrl+C / Esc` | Quit |

#### TUI Options

```bash
--model <MODEL>            # Coder model
--observer-model <MODEL>   # Observer model (defaults to Coder's)
--tool-root <DIR>          # Working directory for exec commands
--auto-observe             # Start with auto-observe ON
```

#### TUI Visual Zones

| Display | Meaning |
|---|---|
| `⠋⠙⠹⠸⠼⠴⠦⠧` spinner | Streaming (200ms/frame) |
| `[iter N/12]` | Agentic tool-call iteration count |
| `[↑N]` | Scrolled N lines above bottom |
| `<think>` block | Dim italic gray (model scratchpad) |
| `[TOOL] cmd` | Yellow bold (command dispatched) |
| `[RESULT] exit=0` | Green (success) |
| `[RESULT] exit=N ⚠` | Red bold (failure — model must diagnose before continuing) |
| `diff --git …` | Blue bold (diff file header) |
| `+` / `-` lines | Green / Red (diff additions / deletions) |
| `@@` lines | Cyan (diff hunk header) |

---

### 5. Getting the Coder to actually create files

1. Set **Tool root** in settings (e.g. `projects/maze-game`)
2. Keep **Edit approval** / **Command approval** ON
3. Tell the Coder:

```text
Create the maze game repo for real. Create folders/files locally. Do it yourself.
```

**What happens:**

```
Coder runs mkdir, git init, file creation in sequence
  ↓
Web GUI: bash code blocks show ▶ run button — click to execute locally
TUI:     exec tool runs automatically → [TOOL]/[RESULT] shown in real time
Lite:    tool calls extracted → approval queue (Approve to run)
  ↓
Results shown inline
```

#### Coder agentic reasoning

Before every tool call, the Coder emits a compact scratchpad:

```
<think>
goal: create directory structure
risk: existing file collision
next: New-Item -ItemType Directory -Force -Path src
</think>
```

This prevents wrong-direction errors (~30 tokens vs 300+ tokens to recover). Also:
- **Output truncation**: stdout capped at 1500 chars, stderr at 600 chars
- **Context pruning**: old tool results collapsed to one-line summaries after 4 turns
- **Error amplification**: on failure, a structured diagnosis prompt is injected before the model continues
- **Max 12 iterations** (infinite loop prevention)

---

### 6. Web GUI Highlights

#### Diff / patch code block highlighting

When the Coder outputs a ` ```diff ` or ` ```patch ` block, it's rendered with per-line colours:

| Line | Colour |
|---|---|
| `diff …` / `index …` | Blue (file header) |
| `+++ ` / `--- ` | White bold (path line) |
| `@@ ` | Cyan (hunk header) |
| `+` lines | Green (addition) |
| `-` lines | Red (deletion) |
| Context lines | Faint white |

#### Other UI features

- **Message timestamps**: relative time per message ("2m ago")
- **`<think>` blocks**: model scratchpad rendered as dim italic monospace, visually separate from prose
- **Proposal expand/collapse**: `toCoder` details hidden by default, revealed with `▶ details`
- **Loop detection**: warning pill + hue shift when Observer repeats itself

#### Diff review mode

Drag & drop a `.diff` or `.patch` file onto the **diff** area in the settings panel (or paste it manually) to feed it directly to Observer for inline code review.

---

### 7. Observer

Observer reads Coder's output from an **independent context** and critiques it. Because it has no implementation pressure, its feedback is more objective.

**Intensity levels:**

| Level | When to use |
|---|---|
| `polite` | Early ideation, fragile state |
| `critical` | Normal development |
| `brutal` | Pre-release, architecture review |

**Reading proposals:**
- **score**: priority 0–100 (≥80 = act now, ≤30 = low priority)
- **phase**: `core` / `feature` / `polish` — which development phase this applies to
- **cost**: implementation effort (`low` / `medium` / `high`)
- **impact**: what improves or gets fixed

Proposals auto-sort by score; phase-mismatched cards are dimmed.

**Loop detection:** If Observer repeats itself, a warning pill appears and the UI applies a hue shift. Add new context or update Coder's output to break the loop.

---

### 8. Security

- Designed for **local use only** (`127.0.0.1`)
- `run_command`, **▶ run**, and TUI exec are powerful — keep approvals enabled
- Threads are stored in browser `localStorage` only (nothing persisted server-side)

---

## Français

### C'est quoi ?

OBSTRAL est un **assistant de développement local qui sépare implémentation, critique et conversation** en rôles distincts.

La plupart des outils LLM mélangent "écrire du code", "revoir du code" et "répondre à des questions" dans la même fenêtre de chat. OBSTRAL divise ces tâches en trois rôles indépendants:

| Panneau | Rôle | Ce qu'il fait |
|---|---|---|
| **Coder** | Implémentation | Écrit du code, exécute des commandes, crée des fichiers |
| **Observer** | Critique | Lit la sortie du Coder et signale risques et améliorations — de manière indépendante |
| **Chat** | Conversation | Discussions de conception, questions, clarification des specs — séparé du contexte d'implémentation |

Chaque rôle peut utiliser un **modèle et un provider différents**.

**Deux interfaces:**

| Interface | Commande | Idéal pour |
|---|---|---|
| **Web GUI** | `cargo run -- serve` | Navigateur, panneau de config, drag & drop diff |
| **TUI** | `cargo run -- tui` | Terminal, SSH, dual-pane ratatui, boucle agentique intégrée |

---

### 1. Quelle version choisir ?

| Environnement | Recommandé |
|---|---|
| **Windows** (surtout pro) | **Python Lite** — mode sûr pour Windows verrouillé (WDAC), stdlib pure |
| **Linux / Mac + navigateur** | **Web GUI** (`cargo run -- serve`) |
| **Terminal / SSH** | **TUI** (`cargo run -- tui`) |

---

### 2. Démarrage rapide

#### Python Lite (recommandé sur Windows)

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# ouvrir http://127.0.0.1:18080/
```

Pour élargir le workspace:
```powershell
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080 -WorkspaceRoot C:\Users\user
```

##### Lite TUI (Python / terminal)

```powershell
cd C:\Users\user\observistral
.\scripts\obstral-lite.ps1 tui --lang fr
# ou:
python .\scripts\obstral_lite_cli.py tui --lang fr
```

#### Web GUI

```bash
cargo run -- serve
# ouvrir http://127.0.0.1:8080/
```

#### TUI

```bash
cargo run -- tui
# ou avec options:
cargo run -- tui --tool-root projects/maze --auto-observe
cargo run -- tui --model gpt-4o --observer-model gpt-4o-mini
```

Si WDAC bloque `obstral.exe`, utilisez Lite.

---

### 3. Clés API

Via le panneau de configuration UI, ou variables d'environnement:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export MISTRAL_API_KEY="..."
export OBS_API_KEY="..."
export GEMINI_API_KEY="..."
```

`Chat`, `Code` et `Observer` peuvent utiliser des providers et modèles différents.

---

### 4. TUI — Interface Terminal

Lancez avec `cargo run -- tui`. Interface dual-pane ratatui complète — pas de navigateur nécessaire.

#### Raccourcis TUI

| Touche | Action |
|---|---|
| `Tab` | Basculer le focus Coder ↔ Observer |
| `Entrée` | Envoyer le message |
| `Shift+Entrée` | Insérer un saut de ligne |
| `Ctrl+K` | Arrêter le streaming |
| `Ctrl+A` | Activer/désactiver l'auto-observation |
| `Ctrl+O` | Déclencher l'Observer manuellement |
| `Ctrl+L` | Effacer le panneau courant |
| `PageUp / PageDown` | Défiler (5 lignes) |
| `Home / End` | Aller au début / à la fin |
| `Ctrl+C / Échap` | Quitter |

#### Zones visuelles TUI

| Affichage | Signification |
|---|---|
| Spinner `⠋⠙…` | Streaming en cours |
| `[iter N/12]` | Itération de l'agent (appels d'outils) |
| Bloc `<think>` | Gris dim italic (raisonnement interne du modèle) |
| `[TOOL] cmd` | Jaune gras (commande envoyée) |
| `[RESULT] exit=0` | Vert (succès) |
| `[RESULT] exit=N ⚠` | Rouge gras (échec — diagnostic requis) |
| Lignes `+` / `-` du diff | Vert / Rouge |
| Ligne `@@` | Cyan (en-tête de hunk) |

---

### 5. Forcer le Coder à créer vraiment des fichiers

1. Réglez **Tool root** (ex: `projects/maze-game`)
2. Laissez **Edit approval** / **Command approval** activés
3. Demandez au Coder:

```text
Crée le repo du jeu de labyrinthe, pour de vrai. Crée les dossiers/fichiers localement. Fais-le toi-même.
```

**Mode Web GUI**: les blocs `bash`/`sh` affichent un bouton **▶ run** — cliquez pour exécuter.
**Mode TUI**: l'outil `exec` s'exécute automatiquement, `[TOOL]`/`[RESULT]` s'affichent en temps réel.
**Mode Lite**: approbation manuelle avant chaque exécution.

#### Raisonnement agentique du Coder

Avant chaque appel d'outil, le Coder émet un scratchpad compact:

```
<think>
goal: créer la structure de répertoires
risk: collision de fichiers existants
next: New-Item -ItemType Directory -Force -Path src
</think>
```

Cela évite les erreurs de "mauvaise direction". Aussi:
- **Troncature de sortie**: stdout limité à 1500 chars, stderr à 600 chars
- **Élagage de contexte**: anciens résultats résumés en une ligne après 4 tours
- **Amplification d'erreur**: en cas d'échec, un prompt de diagnostic est injecté
- **Maximum 12 itérations**

---

### 6. Points forts du Web GUI

#### Coloration des blocs diff / patch

Quand le Coder produit un bloc ` ```diff ` ou ` ```patch `, il est rendu avec des couleurs par ligne:
- **Bleu**: `diff …` / `index …` (en-tête de fichier)
- **Blanc gras**: `+++ ` / `--- ` (chemins)
- **Cyan**: `@@ ` (en-tête de hunk)
- **Vert**: lignes `+` (ajouts)
- **Rouge**: lignes `-` (suppressions)
- **Blanc atténué**: lignes de contexte

#### Autres fonctionnalités UI

- **Horodatage des messages**: temps relatif ("2m ago")
- **Blocs `<think>`**: scratchpad du modèle rendu en monospace italique atténué
- **Expansion des propositions**: détails `toCoder` masqués par défaut (`▶ details` pour afficher)
- **Détection de boucle**: pill d'avertissement + décalage de teinte si l'Observer se répète

#### Mode revue de diff

Glissez-déposez un fichier `.diff` ou `.patch` dans la zone **diff** du panneau de configuration pour le soumettre à l'Observer pour une revue de code.

---

### 7. Observer

L'Observer lit la sortie du Coder depuis un **contexte indépendant** et la critique. Sans pression d'implémentation, ses retours sont plus objectifs.

**Niveaux d'intensité:**

| Niveau | Quand l'utiliser |
|---|---|
| `poli` | Phase d'idéation |
| `critique` | Développement normal |
| `brutal` | Avant release, revue architecture |

**Lire les propositions:**
- **score**: priorité 0–100 (≥80 = agir maintenant)
- **phase**: `core` / `feature` / `polish`
- **cost**: `low` / `medium` / `high`
- **impact**: ce qui s'améliore ou se corrige

Les propositions sont triées par score; les cartes hors-phase sont grisées.

**Détection de boucle:** Si l'Observer se répète, une pill d'avertissement apparaît + décalage de teinte UI. Ajoutez du contexte pour briser la boucle.

---

### 8. Sécurité

- Usage local uniquement (`127.0.0.1`)
- `run_command`, **▶ run** et `exec` TUI sont puissants — gardez les approvals activés
- Les threads sont stockés dans le `localStorage` du navigateur uniquement

---

## License

MIT
