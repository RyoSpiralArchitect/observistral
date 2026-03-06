# OBSTRAL

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-web%20%2B%20TUI-2dd4bf)

> **ひとつのプロンプト窓では足りない。**
> OBSTRALはAIに「第二の脳」を与え、その二つを対立させる。

Languages: [English](README.md) | [日本語](README.ja.md) | [Français](README.fr.md)

---

すべてのAIコーディングツールには同じ問題がある。コードを書いたモデルが、そのコードをレビューする。

それはレビューじゃない。自己弁護だ。

OBSTRALはCoderとObserverを**完全に別のコンテキスト**で動かすことでこれを解決する。ObserverはCoderが書いた1行も「見ていない」。アウトプットしか知らない。だから正直な批評ができる。

---

## なぜOBSTRALか

多くのLLMツールは「会話」を最適化している。
OBSTRALは「制御された実行ループ」を最適化する。役割分離、承認ゲート、そして毎ターンリセットされない累積批評でドリフトを止める。

これはチャットクライアントではない。
開発プロセスの制御エンジンだ。

---

## 3つの役割。3つのコンテキスト。干渉なし。

| 役割 | やること | やらないこと |
|---|---|---|
| **Coder** | 実行 — ファイル操作、シェルコマンド、エージェントループ（最大12回）、5つのツール | 自分のコードを見直すこと |
| **Observer** | 批評 — 提案をスコアリング、スルーした問題をエスカレート | コードに触れること。読むだけ。 |
| **Chat** | 壁打ち — 設計、ゴム鴨、トレードオフ | 実行ループを邪魔すること |

別の役割。望めば別のモデル。コンテキストは常に別。

---

## OBSTRALはあなたが何か言う前から知っている

`tool_root` を設定すると、OBSTRALはプロジェクトを自動スキャンする:

```
[Project Context — auto-detected]
stack: Rust, React (no bundler)
git:   branch=main  modified=2  untracked=1
recent: "fix(observe): require all 4 blocks" · "feat(agent): error classifier"
tree:
  src/          12 files  (Rust source)
  web/           4 files  (JS/CSS)
  scripts/       8 files  (PowerShell)
key:  Cargo.toml · web/app.js · README.md
```

このコンテキストは**最初のプロンプトより前に** Coderのシステムメッセージに注入される。あなたが入力を始める時点で、Coderはすでにスタック、現在のブランチ、変更ファイル、ディレクトリ構造を把握している。

TUIヘッダーにはリアルタイムバッジが表示される: `▸ Rust · React · git:main`
Web UIでは、SettingsのtoolRootフィールドの下にスタックラベルが表示される。

**スタック検出** — OBSTRALはマニフェストファイルを探す:
- `Cargo.toml` → Rust
- `package.json` → Node / React / TypeScript（depsを確認）
- `pyproject.toml` / `requirements.txt` → Python
- `go.mod` → Go
- `pom.xml` → Java

スキャンはセッションごとに1回だけ実行され、200ms以内で完了し、読めないファイルは黙ってスキップする。

---

## OBSTRALが他と違うところ

### Observerには「後ろめたさ」がない

他のツール: 同じモデルがコードを書く → 同じモデルがレビューする → モデルは自分の選択を守る。

OBSTRAL: Observerは毎回フレッシュなコンテキストで動く。「自分ならこう書いた」という記憶がない。見えるのはアウトプットだけ。

結果: より鋭い指摘、正直なリスク評価、言い訳のないフィードバック。

### 提案は消えない

Observerが指摘した問題をスルーすると、提案がエスカレートする:

```
new  →  [UNRESOLVED] +10pt  →  [ESCALATED] +20pt、先頭に固定表示
```

Observerは言ったことを覚えている。`critical` 警告を2回無視すると、それはボードの最上位カードになる。

### exit コードじゃなく、エラーの「種類」を見る

コマンドが失敗したとき、OBSTRALはモデルに生の `exit_code: 1` を渡して終わらない。まず種別を判定する:

| エラー種別 | 注入されるヒント |
|---|---|
| `ENVIRONMENT` | 環境を直せ。ソースコードを触るな。 |
| `SYNTAX` | その1ファイルだけ直せ。関係ないコードを変えるな。 |
| `PATH` | まずパスを確認しろ。確認前に作るな。 |
| `DEPENDENCY` | パッケージをインストールしてからリトライ。 |
| `NETWORK` | サービスとプロキシ変数を確認。 |
| `LOGIC` | ロジックを再読しろ。リランは意味がない。 |

PowerShell注意: エラーを出力しても `exit_code=0` になるケースがある（非終端エラー）。
OBSTRALはこれを `SUSPICIOUS_SUCCESS` として失敗扱いし、偽の前進ドリフトを止める。

### Coderには5つのツールがある

Coderはシェルコマンドだけに限定されない。5つの専用ツールを持っている:

| ツール | 使いどころ |
|---|---|
| `exec(command, cwd?)` | ビルド、テスト、git、パッケージインストール — シェル系全般 |
| `read_file(path)` | シェルのクォーティング問題なしにファイルの正確な内容を読む |
| `write_file(path, content)` | ファイルをアトミックに作成・上書き（親ディレクトリも自動生成） |
| `patch_file(path, search, replace)` | 正確なスニペットを置換 — 曖昧な場合はエラーで止まる |
| `apply_diff(path, diff)` | 統一形式の `@@` diff（複数hunk）を適用 — `patch_file` では小さすぎる編集向け |

`write_file` / `patch_file` / `apply_diff` はテンポラリファイル → リネームのパターンを使うため、書き込み途中でクラッシュしても破損ファイルが残らない。

`patch_file` は検索文字列が**ちょうど1回**だけ存在することを要求する。0回なら修正のためのファイルプレビューを返す。2回以上なら件数をエラーで返す。曖昧さはエラーであり、推測ではない。

**TUIのビジュアルマーカー**でどのツールが動いたか一目でわかる:
- `📄 READ` (teal) — ファイルを読み込み
- `✎ WRITE` (blue) — ファイルを作成・上書き
- `⟳ PATCH` (magenta) — スニペットを置換
- `✓` (緑) / `✗` (赤) — 成功 / エラー

### Coderは自分を疑う

ツールを呼び出す前に、Coderは5行のスクラッチパッドを書く:

```
<think>
goal:   今、何が成功すれば前進するか
risk:   一番ありそうな失敗パターン
doubt:  このアプローチが間違っている可能性   ← 新しいフィールド
next:   具体的なコマンドまたは操作
verify: 成功を確認する方法
</think>
```

`doubt:` フィールドはモデルに「行動前に1つの自己懐疑」を強制する。約50トークン。「自信を持って間違っている」タイプの失敗を防ぐ。

### ステートマシン・ループ（Planning → Executing → Verifying → Recovery）

多くの「エージェントループ」は最大反復回数のタイマーになりがちです。OBSTRALはCoderを小さなステートマシンでルーティングします。

- `planning`  — ゴールと次の具体ステップを言語化する
- `executing` — ツール実行（ファイル/コマンド）
- `verifying` — `goal_check` で「本当に終わったか」を検証してから止まる
- `recovery`  — stuck検出で診断（pwd/ls/git status）と戦略変更を強制する

これにより、長距離ランがREADME整形ループに沈まず、収束しやすくなります。

さらに、直近のツール実行（コマンド + `write_file` / `patch_file` / `apply_diff`）を `[Recent runs]` としてコンパクトに注入し、「やったことを忘れて同じ作業を繰り返す」ループを減らします。

### stop時のゴール検証（偽の「Done」を防ぐ）

モデルが `finish_reason=stop` でツール呼び出しなしに止まった場合でも、OBSTRALは軽いチェック（repo init / tests / build など）を自動実行し、足りない・失敗している場合は `[goal_check]` を差し戻してループを継続させられます。

### @ファイル参照：読み込みターンをスキップ

メッセージに `@path` と書くだけで、そのファイルの内容がプロンプトより前にコンテキストとして注入される:

```
@src/main.rs run_chatは何をしている？
@Cargo.toml @package.json 依存バージョンを並べて比較して
@src/server.rs の400行目のバグを直して
```

TUIはファイルごとに通知を表示する:
```
📎 injected: [src/main.rs] (276 lines, 8192 bytes)
```

Web UIではコンポーザーにチップが表示される（入力中にリアルタイム）:
```
📎 @src/main.rs   📎 @Cargo.toml
```

Coderはファイル内容を即座に参照できる — `read_file` の往復ターンが不要。12イテレーション上限の中で1ターンを節約することが、成功とタイムアウトの差になることがある。

### フェーズゲート: 正しいノイズを黙らせる

`core` / `feature` / `polish` のどのフェーズにいるかをObserverに伝える。マッチしない提案は自動的に暗転する。認証が壊れているときに、CSSの調整で割り込まれなくなる。

### 一目でわかるヘルス

Observerの応答は毎回スコアで終わる:

```
--- health ---
score: 74  rationale: auth is solid, tests cover happy path only
```

❤ **74** → 緑（本番相当ゾーン）。バッジはリアルタイムで更新される。

### 進捗チェックポイント

イテレーション3、6、9で、Coderは止まって自己評価を求められる:

```
1. DONE: planのどのステップが完了済み（exit_code=0確認済み）？
2. REMAINING: 残りは？
3. ON_TRACK: yes/no — noなら次の操作の前にplanを見直せ。
```

「ループで迷走するエージェント」と「迷子になったとき気づくエージェント」の差がここにある。

### Windowsファースト（本当に）

ほとんどのAIコーディングツールはMacで設計され、Linuxでテストされ、Windowsでは「たぶん動く」。

OBSTRALはWindowsで作られた:
- WDACでEXEがブロックされる → Python Liteフォールバックサーバー（標準ライブラリのみ）
- PowerShell構文の自動変換（bash → PS）
- 企業プロキシ環境への対処
- `sh.exe` Win32 error 5 でgit対話プロンプトが壊れる環境

### プラグインレジストリ

フォークせずにOBSTRALを拡張:

```js
registerObserverPlugin({ name: "my-plugin", onProposal, onHealth, onPhase })
registerPhase("security-review", { label: "セキュリティレビュー", color: "#f97316" })
registerValidator(proposals => proposals.filter(p => p.score > 20))
```

`app.js` の前に `<script>` で読み込むだけ。

---

## Observerの出力フォーマット

Observerは自由書きしない。UIがカードにパースする構造化フォーマットで話す:

```
--- phase ---
core

--- proposals ---
title: 入力バリデーション未実装
toCoder: ユーザー入力を受け取る前に長さと文字種をバリデートしてください。
severity: critical
score: 88
phase: core
cost: low
impact: 不正入力によるクラッシュを防止
quote: user_input = input()

--- critical_path ---
入力バリデーションを修正してから次の機能に進んでください。

--- health ---
score: 41  rationale: コアロジックは動くが、インジェクション面が広く開いている
```

すべてのフィールドに意図がある。`quote` は問題の行をカードに固定する。`cost` は詳細を読む前にフィックスの難易度を示す。`phase` は表示制御に使われる。

---

## クイックスタート

**Web UI（推奨）**
```powershell
.\scripts\run-ui.ps1
# → http://127.0.0.1:18080/
```

**TUI（ターミナル）**
```powershell
.\scripts\run-tui.ps1
```

**Headless Coder（CLI）**
```powershell
#（任意）.obstral.md テンプレを生成（stack + test_cmd）
obstral init -C .

# プロジェクト内でコーディングエージェントを実行
obstral agent "fix the failing test" -C . --vibe

# セッションを保存して再開できるようにする（デフォルト: .tmp/obstral_session.json）
obstral agent "fix the failing test" -C . --vibe --session
# 後で再開（プロンプト省略 -> 自動で「続けて」）
obstral agent -C . --vibe --session

# 機械可読な成果物を出力（trace + 最終JSONスナップショット + 実行グラフ）
obstral agent "fix the failing test" -C . --vibe --trace-out .tmp/obstral_trace.jsonl --json-out .tmp/obstral_final.json --graph-out .tmp/obstral_graph.json

# 自動修正ループ（Coder → Observer差分レビュー → Coder）
obstral agent "fix the failing test" -C . --vibe --autofix
obstral agent "fix the failing test" -C . --vibe --autofix 3

# ツール実行を自動承認（プロンプトなし）
obstral agent "fix the failing test" -C . --vibe -y

# 現在のgit diffをObserverでレビュー
obstral review -C .

# チェックポイント以降の差分をレビュー（`obstral agent`が出すhashを指定）
obstral review -C . --base <checkpoint_hash>
 ```

**Python Lite（WDAC / Rustバイナリ不可）**
```powershell
python .\scripts\serve_lite.py
# → http://127.0.0.1:18080/
```

---

## 重要な概念

### tool_root

エージェントの全アクションはワーキングディレクトリ内で実行される。

デフォルト:
- **Web UI**: `.tmp/<thread-id>`（スレッドごとに分離）
- **TUI**: `.tmp/tui_<epoch>`（セッションごとに分離）
- **CLI**: カレントディレクトリ

実際のプロジェクトで作業するには `tool_root` をプロジェクトパスに設定する:
- **TUI**: `-C .` / `--tool-root .` フラグ、または実行中に `/root <path>` スラッシュコマンド
- **Web UI**: Settings → toolRoot フィールド
- **CLI**: `obstral agent "<prompt>" -C .`

`tool_root` が設定されると、OBSTRALは初回使用時にスキャンしてプロジェクトコンテキストを構築する（スタック、git、ツリー）。同セッション内の後続の送信ではスキャンをスキップする。

パストラバーサルはすべてのツール境界でブロックされる: `..` コンポーネントを含むパスはエラーとして拒否される（サイレントではない）。

### 言語

- **UI言語**: TUIは `/lang ja|en|fr`（プロンプトも同時に切り替わる）。
- **Observer言語（Web UI）**: `auto`（デフォルト）は直近のユーザー入力の言語に追従（UIが英語でも日本語批評にできる）。`ui` はUIに追従。`ja`/`en`/`fr` で固定も可能。

### セッション（CLI）

`obstral agent` は `--session[=<path>]` で会話全体（tool call含む）をJSONに保存し、再開できる。

- デフォルトパス: `.tmp/obstral_session.json`
- `-C/--root` 指定時、相対パスの `--session` は `tool_root` からの相対として解釈される
- 実行中も（tool call後などに）自動保存
- プロンプト省略で再開: `obstral agent -C . --session` をもう一度実行
- 最初からやり直し: `--new-session` を付ける（ファイルは上書き）

関連する成果物出力:
- `--trace-out <path>` / `--trace_out`: JSONL trace（tool call / checkpoint / error / done）
- `--json-out <path>` / `--json_out`: 最終セッションスナップショットJSON（messages + tool calls + tool results）
- `--graph-out <path>` / `--graph_out`: 最終メッセージから導出した実行グラフJSON（nodes + edges）
- `-C/--root` 指定時、相対パスは `tool_root` 配下で解決される

セッションJSONにはコードやツール出力が入るので、取り扱い注意。

### 承認（Approvals）

- **Web UI**: edit/command をPendingとしてキューできる。ブラウザから承認・却下。
- **CLI（`obstral agent`）**: `exec` とファイル編集（`write_file` / `patch_file` / `apply_diff`）の前に承認プロンプトが出る。`-y/--yes` または `--no-approvals` でプロンプトを省略。
- **TUI**: 現状はツールを自動承認。

### プロバイダ

OBSTRALはOpenAI互換APIに対応。Mistral、Anthropic、Gemini、ローカルHFモデルも `ChatProvider` traitで差し替え可能。

役割ごとに別モデルを設定できる: Coderのイテレーションには速いモデル、Observerの分析には強力なモデル。よくある実戦エラー: `401`（キー不正）、`429`（レート制限）、`max_tokens` / `max_completion_tokens` のパラメータ差異。

### Chatペルソナ

Chat コンポーザーの上に5つのチップ。セッション中いつでも切り替え可能で、Coder / Observer のペルソナとは完全独立:

| チップ | スタイル |
|---|---|
| 😊 陽気 (cheerful) | 明るく前向きに応答 |
| 🤔 思慮深い (thoughtful) | 前提を確認しながら丁寧に |
| 🧙 師匠 (sensei) | 問いかけで気づかせるスタイル |
| 😏 皮肉屋 (cynical) | 核心を鋭く指摘 |
| 🦆 ゴム鴨 (duck) | 答えを出さず「なぜ？」で思考整理 |

### Chatは相棒（エージェントではない）

Chatはツールを実行しない。実装ランタイム（Coder/Observer）が動いている間の「同居する壁打ち相手」。

Web UIのChatにはオプションが2つある:
- **ランタイム状況を付与**: cwd / 直近エラー / 承認待ち / オープンタスクの要約を小さく注入して、「いま何してる？」がChatだけで成立する。
- **自動タスク化**: 裏でTaskRouterが会話をCoder/Observer向けの具体タスクに変換（**Tasks**に出る）。送るかどうかはユーザーが決める。

### スラッシュコマンド（TUI）

| コマンド | 効果 |
|---|---|
| `/model <name>` | セッション中にモデルを切り替え |
| `/persona <key>` | Coderのペルソナを切り替え |
| `/temp <0.0–1.0>` | temperatureを調整 |
| `/root <path>` | 以降の送信のtool_rootを変更 |
| `/lang ja\|en\|fr` | UI・プロンプト言語を切り替え |
| `/find <query>` | 現在のペインでメッセージをフィルタ |
| `/help` | 全コマンドを表示 |

---

## セキュリティ

デフォルトは `127.0.0.1` のみ。シェル実行は本物 — 承認は有効にしておくこと。

ファイルツールのパスはすべての呼び出しで `tool_root` に対して検証される: `tool_root` の外への絶対パスや `..` コンポーネントはエラーとして拒否される（サイレントではない）。

ネットワークに公開するなら、認証とツール実行の更なるハードニングが必須。

---

## トラブルシュート

**127.0.0.1経由でgithub.comへの接続が失敗する** — 環境変数に死んだプロキシが残っている:
```powershell
Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY,Env:GIT_HTTP_PROXY,Env:GIT_HTTPS_PROXY -ErrorAction SilentlyContinue
```

**対話プロンプトなしでpush** (WDAC / sh.exe Win32 error 5):
```powershell
$env:GITHUB_TOKEN = "ghp_..."
.\scripts\push.ps1
```

**SSH over 443 でpush**（企業ネットワーク）:
```powershell
.\scripts\push_ssh.ps1
```

**obstral.exe のアクセス拒否** — バイナリが実行中:
```powershell
.\scripts\kill-obstral.ps1
```

---

## License

MIT
