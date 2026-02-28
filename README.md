# OBSTRAL (observistral)

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-local%20web-2dd4bf)

Dual-pane "dual brain" coding cockpit: **Coder** (acts) + **Observer** (audits).

> LLM をチャットではなく**実行基盤**として扱う — ファイル作成・コマンド実行・ループ検出まで一気通貫。

Language: [日本語](#日本語) | [English](#english) | [Français](#français)

---

## 日本語

### これは何？

OBSTRAL は、LLM を「チャット」ではなく**実行基盤**として扱うためのローカル UI です。

| ペイン | 役割 |
|---|---|
| **Coder** | コードを書き、コマンドを実行し、ファイルを作る |
| **Observer** | Coder の動きを監視し、リスクを批評し、改善提案を出す |

**何が違うのか:**
- モデルがツール呼び出し非対応でも、出力パターンから「暗黙ツール」を自動抽出して実行
- Observer がループを検出して止める（同じ批評の繰り返しを防ぐ）
- 提案に score / phase / cost が付き、今やるべきことが一目でわかる

---

### 1. どちらを使うか（30秒で決まる）

| 環境 | 推奨 |
|---|---|
| **Windows**（特に会社PC） | **Python Lite** — EXEのWDACブロック回避、Python標準ライブラリのみ |
| **Linux / Mac** | **Rust版** — 高速、バイナリ1本 |

---

### 2. クイックスタート

#### Python Lite（Windows推奨）

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# ブラウザで http://127.0.0.1:18080/ を開く
```

別ディレクトリも操作したい場合（Tool root を広げる）:

```powershell
python .\scripts\serve_lite.py --host 127.0.0.1 --port 18080 --workspace C:\Users\user
```

#### Rust版（Linux / Mac / 管理者権限あり）

```bash
cd observistral
cargo run -- serve
# open http://127.0.0.1:8080/
```

WDACで `obstral.exe` がブロックされる場合:
```powershell
Unblock-File .\target\debug\obstral.exe
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

### 4. Coderにローカル作業をさせる（最重要）

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
bash コードブロックに ▶ run ボタンが表示される
  ↓
クリックすると Tool root 配下でコマンドが実行され、結果がその場に表示
```

モデルがツール呼び出し非対応でも、OBSTRAL が出力から自動抽出:

| 出力パターン | 実行されるツール |
|---|---|
| ファイルパス行 + コードブロック | `write_file` |
| ` ```bash` / ` ```sh` / ` ```shell` ブロック | `run_command` + **▶ run ボタン** |
| ` ```powershell` / ` ```cmd` | `run_command`（Windows向け） |

---

### 5. ワークフロー例：小さな CLI ツールを作る

```
① Coder に「hello という引数を受け取って挨拶する CLI を Python で作って」
② Coder が hello.py と README.md を提案
③ ▶ run で python hello.py World を実行 → 結果が表示
④ Observer が「引数バリデーションがない」と批評 + 提案(score: 72, phase: core)
⑤ 提案を承認 → Coder が修正版を実装
⑥ ▶ run で再テスト
```

---

### 6. Observer を活用する

Observer は Coder の動きを独立した視点で見て批評します。

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

### 7. セキュリティ注意

- **`127.0.0.1` バインド専用**。公開サーバとして使わないでください
- `run_command` / `▶ run` は強力です。デモ中は承認 ON を推奨
- スレッドはブラウザの `localStorage` に保存されます（サーバ側に残りません）

---

## English

### What is this?

OBSTRAL is a **local coding cockpit** where **Coder** executes and **Observer** audits.

> Treat LLMs as an execution substrate, not a chat interface.

---

### 1. Which version to use?

| Environment | Recommendation |
|---|---|
| **Windows** (especially corporate) | **Python Lite** — avoids WDAC exe blocking, pure stdlib |
| **Linux / Mac** | **Rust** — faster, single binary |

---

### 2. Quick Start

#### Python Lite (recommended on Windows)

```powershell
cd C:\Users\user\observistral
.\scripts\run-ui-lite.ps1 -Host 127.0.0.1 -Port 18080
# open http://127.0.0.1:18080/
```

To extend the workspace root:
```powershell
python .\scripts\serve_lite.py --host 127.0.0.1 --port 18080 --workspace C:\Users\user
```

#### Rust

```bash
cargo run -- serve
# open http://127.0.0.1:8080/
```

If WDAC blocks `obstral.exe`, try `Unblock-File .\target\debug\obstral.exe` or use Lite.

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

### 4. Getting the Coder to actually create files

1. Set **Tool root** in settings (e.g. `projects/maze-game`)
2. Keep **Edit approval** / **Command approval** ON
3. Tell the Coder:

```text
Create the maze game repo for real. Create folders/files locally. Do it yourself.
```

Bash/shell code blocks show a **▶ run** button — click to execute locally and see output inline.

Even without native tool calling, OBSTRAL extracts "implied tools" from output patterns:

| Pattern | Tool |
|---|---|
| File path line + fenced code block | `write_file` |
| ` ```bash` / ` ```sh` / ` ```shell` | `run_command` + **▶ run** button |
| ` ```powershell` / ` ```cmd` | `run_command` (Windows) |

---

### 5. Workflow example: building a small CLI tool

```
① Ask Coder: "create a Python CLI that greets a given name"
② Coder writes hello.py and README.md
③ Click ▶ run on the test block → output shown inline
④ Observer critiques: "no input validation" → proposal (score: 72, phase: core)
⑤ Approve → Coder implements fix
⑥ ▶ run again to verify
```

---

### 6. Observer

Observer watches Coder's progress independently and critiques risks.

**Intensity levels:**

| Level | When to use |
|---|---|
| `polite` | Early ideation, fragile state |
| `critical` | Normal development |
| `brutal` | Pre-release, architecture review |

**Reading proposals:**
- **score**: priority 0–100 (≥80 = act now, ≤30 = low priority)
- **phase**: `core` / `feature` / `polish` — which development phase this applies to
- **cost**: implementation effort
- **impact**: what improves or gets fixed

Proposals auto-sort by score, phase-mismatched cards are dimmed.

**Loop detection:** If Observer repeats itself, a warning pill appears and the UI applies a hue shift. Add new context or update Coder's output to break the loop.

---

### 7. Security

- Designed for **local use only** (`127.0.0.1`)
- `run_command` and **▶ run** are powerful — keep approvals enabled
- Threads are stored in browser `localStorage` only

---

## Français

### C'est quoi ?

OBSTRAL est un **cockpit de dev local**: **Coder** exécute, **Observer** audite.

> Traiter les LLM comme une infrastructure d'exécution, pas comme un chat.

---

### 1. Quelle version choisir ?

| Environnement | Recommandé |
|---|---|
| **Windows** (surtout pro) | **Python Lite** — contourne le blocage WDAC, stdlib pure |
| **Linux / Mac** | **Rust** — rapide, binaire unique |

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
python .\scripts\serve_lite.py --host 127.0.0.1 --port 18080 --workspace C:\Users\user
```

#### Rust

```bash
cargo run -- serve
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

`Chat`, `Code` et `Observer` peuvent utiliser des providers différents.

---

### 4. Forcer le Coder à créer vraiment des fichiers

1. Réglez **Tool root** (ex: `projects/maze-game`)
2. Laissez **Edit approval** / **Command approval** activés
3. Demandez au Coder:

```text
Crée le repo du jeu de labyrinthe, pour de vrai. Crée les dossiers/fichiers localement. Fais-le toi-même.
```

Les blocs `bash`/`sh`/`shell` affichent un bouton **▶ run** — cliquez pour exécuter localement.

Même sans tool calling natif, OBSTRAL extrait des "outils implicites":

| Pattern | Outil |
|---|---|
| Chemin + bloc de code | `write_file` |
| Blocs `bash` / `sh` | `run_command` + bouton **▶ run** |
| Blocs `powershell` / `cmd` | `run_command` (Windows) |

---

### 5. Exemple de workflow: un petit CLI

```
① Demander au Coder: "crée un CLI Python qui salue un nom donné"
② Coder écrit hello.py et README.md
③ ▶ run sur le bloc de test → résultat affiché en ligne
④ Observer critique: "pas de validation" → proposition (score: 72, phase: core)
⑤ Approuver → Coder corrige
⑥ ▶ run pour vérifier
```

---

### 6. Observer

**Niveaux d'intensité:**

| Niveau | Quand l'utiliser |
|---|---|
| `丁寧` (poli) | Phase d'idéation |
| `批評` (critique) | Développement normal |
| `容赦なし` (brutal) | Avant release, revue architecture |

**Lire les propositions:**
- **score**: priorité 0–100
- **phase**: `core` / `feature` / `polish`
- **cost**: coût d'implémentation
- **impact**: ce qui s'améliore

Les propositions sont triées par score; les cartes hors-phase sont grisées.

**Détection de boucle:** Si Observer se répète, une pill d'avertissement apparaît + décalage de teinte. Ajoutez du contexte pour briser la boucle.

---

### 7. Sécurité

- Usage local uniquement (`127.0.0.1`)
- `run_command` et **▶ run** sont puissants — gardez les approvals activés
- Les threads sont stockés dans le `localStorage` du navigateur uniquement

---

## License

MIT
