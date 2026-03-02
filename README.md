# OBSTRAL

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-web%20%2B%20TUI-2dd4bf)

**実装・批評・会話** を独立したペインに分けた、ローカル AI 開発アシスタント。

ひとつのチャット窓に「コードを書かせる」「レビューしてもらう」「質問する」を混在させると、コンテキストが汚染され、指摘の質が下がる。OBSTRAL はこれを三役に分離し、それぞれに最適なモデルを割り当てることで開発体験を底上げします。

Language: [日本語](#日本語) | [English](#english) | [Français](#français)

---

## 日本語

### 三役の分離

| ペイン | 役割 | エンジン |
|---|---|---|
| **Coder** | 実装 | ファイル作成・コマンド実行・エージェントループ |
| **Observer** | 批評 | コードレビュー・リスク分析・改善提案のスコアリング |
| **Chat** | 会話 | 設計相談・質疑応答（実装コンテキストと完全に分離） |

それぞれに **別のモデル・別のプロバイダ** を割り当てられます。Observer だけ高性能モデルにする、Chat は軽量モデルにする、といった使い方が可能です。

---

### Coder — エージェントループ

Coder は「コードを書くだけの LLM」ではなく、**シェルコマンドを実行し続けるエージェント**です。

#### 起動から完了まで

```
1. あなたが指示を送る
   ↓
2. Coder が <plan> を出す（初回一回）
   ↓
3. <think> → コマンド実行 → 結果確認 を繰り返す（最大12回）
   ↓
4. 完了を確認してループを終える
```

#### `<plan>` — 最初の一回だけ出るタスク計画

```
<plan>
goal: 迷路ゲームのリポジトリを作成し、動作するプログラムを納品する
steps: 1) ディレクトリ作成  2) main.py 作成  3) ロジック実装  4) 動作確認
risks: 既存ファイルの上書き、パス区切り文字の差異（Windows/Unix）
assumptions: tool_root 配下に書き込み権限がある
</plan>
```

#### `<think>` — コマンド実行ごとに出る4行チェック

```
<think>
goal: src/ ディレクトリを作成する
risk: 既にある場合は -Force オプションで上書き
next: New-Item -ItemType Directory -Path 'src' -Force
verify: Get-Item 'src' で存在確認
</think>
[TOOL] New-Item -ItemType Directory -Path 'src' -Force
[RESULT] exit=0
```

この40トークンのチェックで「間違った方向への突進」（復帰コスト300〜500トークン）を防ぎます。

#### エラー時のプロトコル

- `exit_code ≠ 0` → **即停止**。エラー内容を引用して原因を特定してから修正
- 同じアプローチが3回連続失敗 → **戦略を変える**（同じコマンドをリトライしない）

#### コンテキスト管理

| 機能 | 内容 |
|---|---|
| **コンテキスト剪定** | 古いツール結果を1行要約に折りたたみ（直近4ターン保持） |
| **出力トランケーション** | stdout 1500文字 / stderr 600文字で自動カット |
| **最大反復回数** | 12回でキャップ（無限ループ防止） |

---

### Observer — 批評と提案

Observer は Coder の出力を**独立したコンテキスト**で読みます。実装に関与していないため、指摘が客観的になります。

#### 5軸レビュー

毎回、次の5軸でリスクを洗い出します: **正しさ・セキュリティ・信頼性・性能・保守性**

#### 提案フォーマット

Observer の出力末尾に `--- proposals ---` ブロックが自動パースされ、カード形式で表示されます:

```
--- proposals ---
title: 入力バリデーションが未実装
toCoder: ユーザー入力を受け取る前に長さと文字種をバリデートしてください。
severity: critical
score: 88
phase: core
cost: low
impact: バッファオーバーフロー・クラッシュを防止
quote: user_input = input()
```

| フィールド | 意味 |
|---|---|
| **score** | 優先度 0–100（≥80 = 今すぐ対応、≤30 = 後回し可） |
| **phase** | `core` / `feature` / `polish` — 今の開発フェーズと一致しない提案は暗転 |
| **cost** | `low` / `medium` / `high` — 実装コストの目安 |
| **severity** | `critical` / `warn` / `info` — カードの色分けに使用 |
| **toCoder** | Coder に直接送れる具体的な指示文 |

提案はスコア順に自動ソートされます。**「Send to Coder」ボタン**を押すと確認ダイアログ経由で Coder のコンテキストに送信されます。

#### 自動実況モード（Auto-observe）

`Ctrl+A`（TUI）または設定の **Auto-observe** をONにすると、Coder がレスポンスを出すたびに Observer が自動でレビューを開始します。

---

### UX のポイント

#### デュアルペイン（ドラッグでリサイズ）

Coder と Observer のペイン境界をドラッグして比率を自由に変更できます（20%〜80%）。設定はブラウザに記憶されます。

#### Observer サブタブ

Observer ペインには **分析**（提案カード）と **チャット**（直接対話）の2タブがあります。Coder の実装をブロックせずに Observer と会話できます。

#### `<think>` ブロックの視覚表示

モデルの推論スクラッチパッドは薄灰色 italic でレンダリングされ、実際の出力と区別されます。

```
<think>          ← 薄灰色 italic で表示
goal: ...
...
</think>
[TOOL] ...       ← 黄色太字
[RESULT] exit=0  ← 緑
```

#### 提案スコアバー

各提案にスコアバーが表示されます: 緑（≥70）→ 黄（≥40）→ 赤（<40）

#### ループ検出

Observer が同じ批評を繰り返し始めると:
- 警告 pill（`Loop detected ×N`）がヘッダに表示
- UI 全体が色相シフト

Observer の入力欄に新しい観点を送るか、Coder の成果物を更新して解消します。

#### diff レビューモード

設定の **diff** エリアにファイルをドラッグ&ドロップ（または貼り付け）すると、Observer に差分を渡してコードレビューさせられます。diff コードブロックは行単位で色付けされます（青/白/シアン/緑/赤）。

#### ファイルチップ

Coder のメッセージからパスを自動抽出し、クリックで開けるチップとして表示します。

---

### セットアップ

#### どれを使うか（30秒）

| 環境 | 推奨 |
|---|---|
| **Windows**（特に会社PC） | **Python Lite** — WDACブロック回避、Python標準ライブラリのみ |
| **Linux / Mac + ブラウザ** | **Web GUI** (`cargo run -- serve`) |
| **ターミナル / SSH** | **TUI** (`cargo run -- tui`) |

#### クイックスタート

**Python Lite（Windows推奨）**
```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# http://127.0.0.1:18080/
```

**Web GUI**
```bash
cargo run -- serve   # http://127.0.0.1:8080/
```

**TUI**
```bash
cargo run -- tui
cargo run -- tui --tool-root projects/maze --auto-observe
cargo run -- tui --model gpt-4o --observer-model gpt-4o-mini
```

WDACで `obstral.exe` がブロックされる場合（管理者 PowerShell）:
```powershell
Add-MpPreference -ExclusionPath "C:\Users\user\observistral\target"
```

#### APIキー

UIの設定パネルに直接入力、または環境変数:

```powershell
$env:OPENAI_API_KEY    = "sk-..."
$env:ANTHROPIC_API_KEY = "sk-ant-..."
$env:MISTRAL_API_KEY   = "..."
$env:OBS_API_KEY       = "..."   # OpenAI互換 (vLLM / LM Studio等)
$env:GEMINI_API_KEY    = "..."
```

---

### TUI リファレンス

```
┌─ OBSTRAL ─────────────────────────────────────────────────────────┐
│ C:gpt-4o-mini  O:gpt-4o-mini  Tab=切替  Ctrl+A=自動  Ctrl+K=停止 │
├─────────────────────────────┬──────────────────────────────────────┤
│ ◉ CODER ⠸ [iter 3/12]       │ ○ OBSERVER                          │
│                             │                                      │
│ <plan>                      │ obs › 入力バリデーションが抜けている │
│ goal: maze game 完成         │                                      │
│ steps: 1)dir 2)main 3)test  │ ── proposals ──────────────────────  │
│ </plan>                     │ ████████░░ 88pt  critical            │
│                             │ 入力バリデーション未実装             │
│ <think>                     │ [Send to Coder]                      │
│ goal: src/ 作成             │                                      │
│ next: New-Item -Force       │ ████░░░░░░ 42pt  warn                │
│ </think>                    │ エラー時のメッセージが不明瞭         │
│ [TOOL] New-Item -ItemType…  │                                      │
│ [RESULT] exit=0             │                                      │
├─────────────────────────────┴──────────────────────────────────────┤
│ › CODER  Enter=送信  Shift+Enter=改行                              │
│ > _                                                                │
└────────────────────────────────────────────────────────────────────┘
```

#### キーバインド

| キー | アクション |
|---|---|
| `Tab` | Coder ↔ Observer フォーカス切り替え |
| `Enter` | メッセージ送信 |
| `Shift+Enter` | 改行 |
| `Ctrl+K` | ストリーミング停止 |
| `Ctrl+A` | 自動実況 ON/OFF |
| `Ctrl+O` | Observer を手動トリガー |
| `Ctrl+L` | 現在ペインのメッセージクリア |
| `PageUp / PageDown` | スクロール（5行） |
| `Home / End` | 先頭 / 最下部へジャンプ |
| `Ctrl+C / Esc` | 終了 |

#### TUI 表示凡例

| 表示 | 意味 |
|---|---|
| `⠋⠙⠹⠸` スピナー | ストリーミング中 |
| `[iter N/12]` | エージェントのツール呼び出し回数 |
| `[↑N]` | N行上にスクロール中 |
| `<think>` / `<plan>` | 薄灰色 italic（推論スクラッチパッド） |
| `[TOOL] cmd` | 黄色太字（実行コマンド） |
| `[RESULT] exit=0` | 緑（成功） |
| `[RESULT] exit=N ⚠` | 赤太字（失敗 — 診断必須） |
| `diff --git …` | 青太字（diff ヘッダ） |
| `+` / `-` 行 | 緑 / 赤（diff 追加 / 削除） |

#### TUI オプション

```bash
--model <MODEL>            # Coder モデル
--observer-model <MODEL>   # Observer モデル（省略時は Coder と同じ）
--tool-root <DIR>          # exec コマンドの作業ディレクトリ
--auto-observe             # 起動時から自動実況 ON
```

---

### セキュリティ

- **`127.0.0.1` バインド専用**。公開サーバとして使わないでください
- `run_command` / `▶ run` / TUI exec は強力です。承認 ON を推奨
- スレッドはブラウザの `localStorage` にのみ保存されます

---

## English

### Three Roles, Three Contexts

| Pane | Role | What it does |
|---|---|---|
| **Coder** | Implementation | File creation, command execution, agentic loop |
| **Observer** | Critique | Code review, risk analysis, scored improvement proposals |
| **Chat** | Conversation | Design discussion, Q&A — fully isolated from implementation context |

Each role can use a **different model and provider**. Point Observer at a powerful reasoning model, use a fast model for Chat — it's all configurable per pane.

---

### Coder — Agentic Loop

The Coder isn't just an LLM that writes code. It's an **autonomous agent that executes shell commands in a loop** until the task is complete.

#### Flow

```
1. You send a task
   ↓
2. Coder outputs <plan> (once, at the start)
   ↓
3. <think> → execute → verify — repeats up to 12 times
   ↓
4. Coder confirms completion and stops
```

#### `<plan>` — Task plan, emitted once

```
<plan>
goal: create a working maze game repository
steps: 1) create dirs  2) write main.py  3) implement logic  4) verify run
risks: existing file collision, Windows/Unix path separator differences
assumptions: write access under tool_root
</plan>
```

#### `<think>` — 4-line check before every command

```
<think>
goal: create the src/ directory
risk: already exists — use -Force flag
next: New-Item -ItemType Directory -Path 'src' -Force
verify: Get-Item 'src' to confirm existence
</think>
[TOOL] New-Item -ItemType Directory -Path 'src' -Force
[RESULT] exit=0
```

This ~40-token check prevents wrong-direction mistakes that cost 300–500 tokens to recover from.

#### Error protocol

- `exit_code ≠ 0` → **stop immediately**. Quote the error, identify root cause, fix with one corrected command.
- Same approach fails 3 times in a row → **abandon and try a different strategy**.

#### Context management

| Feature | Details |
|---|---|
| **Context pruning** | Old tool results folded to one-line summaries (last 4 turns kept) |
| **Output truncation** | stdout capped at 1500 chars, stderr at 600 chars |
| **Iteration cap** | 12 max (prevents infinite loops) |

---

### Observer — Critique and Proposals

Observer reads Coder's output from an **independent context**. No implementation pressure means more objective feedback.

#### Five-axis review

Every review covers: **correctness · security · reliability · performance · maintainability**

#### Proposal format

Observer's output is automatically parsed for a `--- proposals ---` block and displayed as scored cards:

```
--- proposals ---
title: Input validation missing
toCoder: Validate user input for length and character type before processing.
severity: critical
score: 88
phase: core
cost: low
impact: prevents buffer overflow and crash on malformed input
quote: user_input = input()
```

| Field | Meaning |
|---|---|
| **score** | Priority 0–100 (≥80 = act now, ≤30 = low priority) |
| **phase** | `core` / `feature` / `polish` — proposals that don't match current phase are dimmed |
| **cost** | `low` / `medium` / `high` — implementation effort |
| **severity** | `critical` / `warn` / `info` — drives card colour |
| **toCoder** | A ready-to-send instruction for the Coder |

Proposals auto-sort by score. **"Send to Coder"** routes the proposal through a confirmation dialog directly into the Coder's context.

#### Auto-observe

`Ctrl+A` (TUI) or **Auto-observe** in settings fires Observer automatically after every Coder response.

---

### UX Highlights

#### Drag-to-resize panes

Drag the divider between Coder and Observer to any ratio from 20%–80%. Saved in localStorage.

#### Observer subtabs

The Observer pane has two tabs: **Analysis** (proposal cards) and **Chat** (direct conversation). Talk to Observer without interrupting the Coder.

#### `<think>` block rendering

The model's scratchpad is rendered in dim italic monospace — visually distinct from code and prose output.

```
<think>          ← dim italic grey
goal: ...
...
</think>
[TOOL] ...       ← yellow bold
[RESULT] exit=0  ← green
```

#### Proposal score bars

Each proposal card shows a colour-coded score bar: green (≥70) → amber (≥40) → red (<40).

#### Loop detection

If Observer starts repeating the same critique:
- A `Loop detected ×N` warning pill appears in the Observer header
- The UI applies a hue shift to signal the deadlock

Send new context to Observer or update Coder's output to break it.

#### Diff review mode

Drag & drop a `.diff` or `.patch` file onto the **diff** area in settings (or paste it) to feed it directly to Observer for inline code review.

#### File chips

Paths mentioned in Coder messages are extracted and shown as clickable chips for quick file access.

---

### Setup

#### Which version?

| Environment | Recommendation |
|---|---|
| **Windows** (especially corporate) | **Python Lite** — bypasses WDAC, pure Python stdlib |
| **Linux / Mac + browser** | **Web GUI** (`cargo run -- serve`) |
| **Terminal / SSH** | **TUI** (`cargo run -- tui`) |

#### Quick start

**Python Lite (Windows)**
```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# open http://127.0.0.1:18080/
```

**Web GUI**
```bash
cargo run -- serve   # http://127.0.0.1:8080/
```

**TUI**
```bash
cargo run -- tui
cargo run -- tui --tool-root projects/maze --auto-observe
cargo run -- tui --model gpt-4o --observer-model gpt-4o-mini
```

If WDAC blocks `obstral.exe` (admin PowerShell):
```powershell
Add-MpPreference -ExclusionPath "C:\Users\user\observistral\target"
```

#### API Keys

Set in the UI settings panel, or via environment variables:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export MISTRAL_API_KEY="..."
export OBS_API_KEY="..."      # OpenAI-compatible (vLLM, LM Studio, etc.)
export GEMINI_API_KEY="..."
```

`Chat`, `Code`, and `Observer` each accept a different provider and model.

---

### TUI Reference

```
┌─ OBSTRAL ─────────────────────────────────────────────────────────┐
│ C:gpt-4o-mini  O:gpt-4o-mini  Tab=switch  Ctrl+A=auto  Ctrl+K=stop│
├─────────────────────────────┬──────────────────────────────────────┤
│ ◉ CODER ⠸ [iter 3/12]       │ ○ OBSERVER                          │
│                             │                                      │
│ <plan>                      │ obs › input validation missing       │
│ goal: maze game done        │                                      │
│ steps: 1)dir 2)main 3)test  │ ── proposals ──────────────────────  │
│ </plan>                     │ ████████░░ 88pt  critical            │
│                             │ Input validation missing             │
│ <think>                     │ [Send to Coder]                      │
│ goal: create src/           │                                      │
│ next: New-Item -Force       │ ████░░░░░░ 42pt  warn                │
│ </think>                    │ Error messages are unclear           │
│ [TOOL] New-Item -ItemType…  │                                      │
│ [RESULT] exit=0             │                                      │
├─────────────────────────────┴──────────────────────────────────────┤
│ › CODER  Enter=send  Shift+Enter=newline                           │
│ > _                                                                │
└────────────────────────────────────────────────────────────────────┘
```

#### Key Bindings

| Key | Action |
|---|---|
| `Tab` | Switch focus Coder ↔ Observer |
| `Enter` | Send message |
| `Shift+Enter` | Insert newline |
| `Ctrl+K` | Stop streaming |
| `Ctrl+A` | Toggle auto-observe |
| `Ctrl+O` | Trigger Observer manually |
| `Ctrl+L` | Clear current pane |
| `PageUp / PageDown` | Scroll 5 lines |
| `Home / End` | Jump to top / bottom |
| `Ctrl+C / Esc` | Quit |

#### Visual Zones

| Display | Meaning |
|---|---|
| `⠋⠙⠹⠸` spinner | Streaming |
| `[iter N/12]` | Agent tool-call count |
| `[↑N]` | Scrolled N lines above bottom |
| `<think>` / `<plan>` | Dim italic (model scratchpad) |
| `[TOOL] cmd` | Yellow bold (command dispatched) |
| `[RESULT] exit=0` | Green (success) |
| `[RESULT] exit=N ⚠` | Red bold (failure — must diagnose) |
| `diff --git …` | Blue bold |
| `+` / `-` lines | Green / Red |

#### Options

```bash
--model <MODEL>            # Coder model
--observer-model <MODEL>   # Observer model (defaults to Coder's)
--tool-root <DIR>          # Working directory for exec commands
--auto-observe             # Start with auto-observe ON
```

---

### Security

- **Local use only** (`127.0.0.1`) — do not expose to a network
- `run_command`, **▶ run**, and TUI exec are powerful — keep approvals enabled
- Threads are stored in browser `localStorage` only (nothing on the server)

---

## Français

### Trois rôles, trois contextes

| Panneau | Rôle | Ce qu'il fait |
|---|---|---|
| **Coder** | Implémentation | Création de fichiers, exécution de commandes, boucle agentique |
| **Observer** | Critique | Revue de code, analyse de risques, propositions scorées |
| **Chat** | Conversation | Questions, conception — isolé du contexte d'implémentation |

Chaque rôle peut utiliser un **modèle et un provider différents**.

---

### Coder — Boucle agentique

Le Coder n'est pas un simple LLM qui écrit du code. C'est un **agent autonome qui exécute des commandes shell en boucle** jusqu'à la fin de la tâche.

#### Déroulement

```
1. Vous envoyez une tâche
   ↓
2. Le Coder produit <plan> (une fois, au départ)
   ↓
3. <think> → exécution → vérification — jusqu'à 12 fois
   ↓
4. Le Coder confirme et s'arrête
```

#### `<plan>` — Planification initiale (une fois)

```
<plan>
goal: créer un jeu de labyrinthe fonctionnel
steps: 1) créer les dirs  2) écrire main.py  3) implémenter  4) vérifier
risks: collision de fichiers existants, séparateurs Windows/Unix
assumptions: droits d'écriture sous tool_root
</plan>
```

#### `<think>` — Vérification en 4 lignes avant chaque commande

```
<think>
goal: créer le répertoire src/
risk: existe déjà — utiliser -Force
next: New-Item -ItemType Directory -Path 'src' -Force
verify: Get-Item 'src' pour confirmer
</think>
[TOOL] New-Item -ItemType Directory -Path 'src' -Force
[RESULT] exit=0
```

Ce contrôle de ~40 tokens évite les erreurs de "mauvaise direction" (300–500 tokens à corriger).

#### Protocole d'erreur

- `exit_code ≠ 0` → **arrêt immédiat**. Citer l'erreur, identifier la cause, corriger.
- Même approche échoue 3 fois → **changer complètement de stratégie**.

#### Gestion du contexte

| Fonction | Détail |
|---|---|
| **Élagage de contexte** | Anciens résultats résumés en une ligne (4 derniers tours conservés) |
| **Troncature de sortie** | stdout 1500 chars, stderr 600 chars |
| **Cap d'itérations** | 12 maximum |

---

### Observer — Critique et propositions

L'Observer lit la sortie du Coder depuis un **contexte indépendant**. Sans pression d'implémentation, les retours sont plus objectifs.

#### Revue sur 5 axes

Chaque revue couvre: **exactitude · sécurité · fiabilité · performance · maintenabilité**

#### Format des propositions

La sortie de l'Observer est parsée pour un bloc `--- proposals ---` et affichée en cartes:

```
--- proposals ---
title: Validation des entrées manquante
toCoder: Validez la longueur et le type des entrées utilisateur avant traitement.
severity: critical
score: 88
phase: core
cost: low
impact: prévient les crashs sur entrées malformées
quote: user_input = input()
```

Les propositions sont triées par score. **"Send to Coder"** envoie la proposition dans le contexte du Coder via une confirmation.

#### Auto-observation

`Ctrl+A` (TUI) ou **Auto-observer** dans les paramètres déclenche l'Observer automatiquement après chaque réponse du Coder.

---

### Points forts UX

#### Redimensionnement des panneaux

Faites glisser le séparateur entre Coder et Observer pour ajuster le ratio (20 %–80 %). Mémorisé dans le localStorage.

#### Sous-onglets de l'Observer

Le panneau Observer a deux onglets : **Analyse** (cartes de propositions) et **Chat** (conversation directe). Dialoguez avec l'Observer sans interrompre le Coder.

#### Rendu des blocs `<think>`

Le scratchpad du modèle est rendu en monospace italique atténué — visuellement distinct du code et de la prose.

#### Barres de score des propositions

Chaque carte affiche une barre colorée: vert (≥70) → ambre (≥40) → rouge (<40).

#### Détection de boucle

Si l'Observer répète la même critique:
- Pill `Loop detected ×N` dans l'en-tête
- Décalage de teinte sur l'interface

Envoyez un nouveau contexte à l'Observer ou mettez à jour la sortie du Coder.

#### Mode revue de diff

Glissez un `.diff` ou `.patch` dans la zone **diff** des paramètres pour le soumettre à l'Observer.

---

### Installation

#### Quelle version ?

| Environnement | Recommandé |
|---|---|
| **Windows** (pro) | **Python Lite** — contourne WDAC, stdlib pure |
| **Linux / Mac + navigateur** | **Web GUI** (`cargo run -- serve`) |
| **Terminal / SSH** | **TUI** (`cargo run -- tui`) |

#### Démarrage rapide

**Python Lite (Windows)**
```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
```

**Web GUI**
```bash
cargo run -- serve   # http://127.0.0.1:8080/
```

**TUI**
```bash
cargo run -- tui
cargo run -- tui --tool-root projects/maze --auto-observe
```

#### Clés API

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export MISTRAL_API_KEY="..."
export OBS_API_KEY="..."
export GEMINI_API_KEY="..."
```

---

### TUI — Raccourcis

| Touche | Action |
|---|---|
| `Tab` | Coder ↔ Observer |
| `Entrée` | Envoyer |
| `Ctrl+A` | Auto-observation ON/OFF |
| `Ctrl+O` | Déclencher l'Observer |
| `Ctrl+K` | Arrêter le streaming |
| `Ctrl+L` | Effacer le panneau |
| `PageUp/Down` | Défiler |
| `Ctrl+C / Échap` | Quitter |

---

### Sécurité

- Usage local uniquement (`127.0.0.1`)
- Gardez les approbations activées pour `run_command` / exec
- Threads stockés dans le `localStorage` du navigateur uniquement

---

## License

MIT
