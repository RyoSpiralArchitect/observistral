# observistral

プロバイダ抽象化付きチャットbot実行基盤です。以下に対応しています。

- OpenAI互換API（OpenAI本家、各種互換エンドポイント）
- Anthropic Messages API
- Hugging Face ローカル/オフライン推論（`transformers`）

## インストール

```bash
pip install -e .
# HFローカルを使う場合
pip install -e .[hf]
```

## CLI

```bash
observistral "この機能の設計を壁打ちしたい" \
  --mode 壁打ち \
  --persona thoughtful \
  --provider openai-compatible \
  --model gpt-4o-mini \
  --api-key "$OBS_API_KEY"
```

### モード

- `実況`
- `壁打ち`
- `diff批評`

### ペルソナ（切り替えUI / CLI）

- `default` (Balanced)
- `novelist`
- `cynical`
- `cheerful`
- `thoughtful`

```bash
# ペルソナ一覧
observistral --list-personas

# 例: 小説家ペルソナ
observistral "短い導入文を書いて" --mode 実況 --persona novelist --provider openai-compatible --model gpt-4o-mini --api-key "$OBS_API_KEY"
```

### さらに進めた実装ポイント

- `--diff-file` でパッチ/差分ファイルを読み込んで `diff批評` に自動注入
- `--stdin` で標準入力をプロンプトへ追加
- `--list-providers` で利用可能なプロバイダ別名を表示
- `--list-personas` で利用可能なペルソナを表示
- OpenAI互換はAPIキーなしでも利用可能（ローカル推論サーバー向け）
- AnthropicはAPIキー必須
- HFローカルは `OBS_HF_LOCAL_ONLY=1` でオフライン優先

### プロバイダ切替例

```bash
# OpenAI互換
observistral "こんにちは" --provider openai-compatible --model gpt-4o-mini --api-key "$OBS_API_KEY"

# OpenAI互換（ローカルvLLM/LM Studioなど）
observistral "こんにちは" --provider openai-compatible --model local-model --base-url "http://localhost:8000/v1"

# Anthropic
observistral "こんにちは" --provider anthropic --model claude-3-5-sonnet-latest --api-key "$ANTHROPIC_API_KEY"

# HFローカル（オフライン）
OBS_HF_LOCAL_ONLY=1 observistral "READMEを要約" --provider hf --model mistralai/Mistral-7B-Instruct-v0.2

# diff批評（diffファイル読み込み）
observistral "この差分をレビューして" --mode diff批評 --persona thoughtful --provider openai-compatible --model gpt-4o-mini --api-key "$OBS_API_KEY" --diff-file ./changes.diff
```

環境変数でも指定できます。

- `OBS_PROVIDER`
- `OBS_MODEL`
- `OBS_API_KEY`
- `OBS_BASE_URL`
- `OBS_TIMEOUT_SECONDS`
- `OBS_HF_DEVICE`
- `OBS_HF_LOCAL_ONLY`
- `OBS_PERSONA`

---

## Français (FR)

`observistral` est un runtime de chatbot avec abstraction de fournisseurs.

Fonctionnalités principales :
- Compatible OpenAI (API `/chat/completions`)
- Support Anthropic (`/v1/messages`)
- Support local/offline Hugging Face (`transformers`)
- Changement de modèle/fournisseur via options CLI ou variables d'environnement
- Modes prêts à l'emploi : `実況`, `壁打ち`, `diff批評`
- Personas prêtes à l'emploi : `novelist`, `cynical`, `cheerful`, `thoughtful`

Exemple rapide :

```bash
observistral "Aide-moi à structurer cette idée" \
  --mode 壁打ち \
  --persona thoughtful \
  --provider openai-compatible \
  --model gpt-4o-mini \
  --api-key "$OBS_API_KEY"
```

Lister les personas :

```bash
observistral --list-personas
```

Revue de patch :

```bash
observistral "Critique ce diff" \
  --mode diff批評 \
  --persona cynical \
  --provider anthropic \
  --model claude-3-5-sonnet-latest \
  --api-key "$ANTHROPIC_API_KEY" \
  --diff-file ./changes.diff
```

SpiralReality
