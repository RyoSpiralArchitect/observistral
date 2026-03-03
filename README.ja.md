# OBSTRAL

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-web%20%2B%20TUI-2dd4bf)

> **ひとつのチャット窓は足りない。**
> OBSTRALはAIに「第二の脳」を与え、その二つを対立させる。

Languages: [English](README.md) | [日本語](README.ja.md) | [Français](README.fr.md)

---

すべてのAIコーディングツールには同じ問題がある。コードを書いたモデルが、そのコードをレビューする。

それはレビューじゃない。自己弁護だ。

OBSTRALはCoderとObserverを**完全に別のコンテキスト**で動かすことでこれを解決する。ObserverはCoderが書いた1行も「見ていない」。アウトプットしか知らない。だから正直な批評ができる。

---

## 3つの役割。3つのコンテキスト。干渉なし。

| 役割 | やること | やらないこと |
|---|---|---|
| **Coder** | 実行 — ファイル操作、シェルコマンド、エージェントループ（最大12回） | 自分のコードを見直すこと |
| **Observer** | 批評 — 提案をスコアリング、スルーした問題をエスカレート | コードに触れること。読むだけ。 |
| **Chat** | 壁打ち — 設計、ゴム鴨、トレードオフ | 実行ループを邪魔すること |

別の役割。望めば別のモデル。コンテキストは常に別。

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

### Coderは自分を疑う

コマンドを実行する前に、Coderは5行のスクラッチパッドを書く:

```
<think>
goal:   今、何が成功すれば前進するか
risk:   一番ありそうな失敗パターン
doubt:  このアプローチが間違っている可能性   ← 新しいフィールド
next:   具体的なコマンド
verify: 成功を確認する方法
</think>
```

`doubt:` フィールドはモデルに「行動前に1つの自己懐疑」を強制する。約50トークン。「自信を持って間違っている」タイプの失敗を防ぐ。

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
3. ON_TRACK: yes/no — noなら次のコマンドの前にplanを見直せ。
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

**Python Lite（WDAC / Rustバイナリ不可）**
```powershell
python .\scripts\serve_lite.py
# → http://127.0.0.1:18080/
```

---

## 重要な概念

### tool_root

エージェントの全アクションはスクラッチディレクトリ内で実行される。デフォルト: `.tmp/<thread-id>`。

ネストしたgitリポジトリ、プロジェクトルートへの迷い込みファイル、「なぜ間違ったディレクトリで実行された？」という典型的な失敗を防ぐ。スレッドごとに完全隔離。

### 承認（Approvals）

- **Edit approval**: `write_file` 呼び出しはPending Editsとしてキューに積まれる。一つずつ承認・却下できる。
- **Command approval**: `exec` 呼び出しも同様にゲートできる（任意）。Coderはあなたの判断を待って再開する。

どちらも作業を止めずに静かにキューに積まれる。

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

---

## セキュリティ

デフォルトは `127.0.0.1` のみ。シェル実行は本物 — 承認は有効にしておくこと。

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
