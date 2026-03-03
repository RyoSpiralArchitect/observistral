(() => {
  "use strict";

  const root = document.getElementById("app-root");
  if (!root) return;

  const e = React.createElement;
  const { useCallback, useEffect, useMemo, useRef, useState } = React;

  // ╔══════════════════════════════════════════════════════════╗
  // ║  SECTION: Constants & i18n                               ║
  // ╚══════════════════════════════════════════════════════════╝
  const LS = {
    lang: "obstral.lang.v1",
    config: "obstral.config.v1",
    threads: "obstral.threads.v1",
    active: "obstral.active.v1",
    splitPct: "obstral.splitPct.v1",
  };

  // ── i18n strings (en / ja / fr) ──────────────────────────────────────────────
  const I18N = {
    en: {
      threads: "Threads",
      newThread: "New",
      rename: "Rename",
      del: "Delete",
      delConfirm: "Delete this thread?",
      settings: "Settings",
      presets: "Presets",
      chat: "Chat",
      coder: "Coder",
      observer: "Observer",
      proposals: "Proposals",
      tasks: "Tasks",
      planningTasks: "Planning tasks…",
      sendToCoder: "Send to coder",
      sendToObserver: "Send to observer",
      done: "Done",
      approved: "approved",
      alreadyApproved: "Already approved",
      phaseMismatch: "Phase mismatch",
      phaseMismatchConfirm: "Phase mismatch (current: {cur} / proposal: {prop}). Send anyway?",
      applyMeta: "Apply",
      includeCoderContext: "Include coder context",
      insertCliTemplate: "CLI template",
      editApproval: "Edit approval",
      commandApproval: "Command approval",
      autoObserve: "Auto-observe",
      observerHint: "Type a message below to start observing, or enable Auto-observe in settings.",
      forceAgent: "Agent mode",
      toolRoot: "Tool root",
      workdir: "Workdir",
      findInThread: "Find in thread…",
      noMatches: "No matches",
      pendingEdits: "Pending edits",
      approve: "Approve",
      reject: "Reject",
      send: "Send",
      stop: "Stop",
      exportMd: "Markdown",
      clear: "Clear",
      provider: "Provider",
      model: "Model",
      chatModel: "Chat model",
      codeModel: "Code model",
      fetchModels: "Fetch models",
      baseUrl: "Base URL",
      codeProvider: "Code provider",
      codeBaseUrl: "Code Base URL",
      observerProvider: "Observer provider",
      observerBaseUrl: "Observer Base URL",
      observerModel: "Observer model",
      apiKey: "API key",
      apiKeyChat: "Chat API key",
      apiKeyCode: "Code API key",
      apiKeyObserver: "Observer API key",
      sameAsChat: "(same as chat)",
      mode: "Mode",
      persona: "Persona",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "Streaming",
      cot: "CoT",
      brief: "Brief",
      structured: "Structured",
      deep: "Deep",
      on: "ON",
      off: "OFF",
      diff: "diff",
      diffHint: "Auto-injected in diff review mode",
      placeholder: "Type here. Enter to send (Shift+Enter for newline)",
      ready: "ready",
      sending: "sending…",
      streaming: "streaming…",
      error: "error",
      fetchFailed: "Failed to fetch (is OBSTRAL serve running?)",
      copy: "Copy",
      reader: "Read",
      refresh: "Refresh",
      keys: "API keys",
      loopDetected: "Loop detected",
      observerIntensity: "Observer intensity",
      polite: "polite",
      critical: "critical",
      brutal: "brutal",
      vibeAgent: "Vibe agent",
      vibeMaxTurns: "Vibe max turns",
      details: "details",
      hide: "hide",
      serverOutdated: "Server outdated",
      more: "More",
      less: "Less",
      shortcuts: "Keyboard shortcuts",
      close: "Close",
      delQ: "Delete?",
      yes: "Yes",
      no: "No",
      sendMsg: "Send message",
      newline: "New line",
      toggleHelp: "Toggle this help",
      closeModals: "Close modals / cancel rename",
      stopStreamingCoder: "Stop streaming (Coder/Observer)",
    },
    ja: {
      threads: "スレッド",
      newThread: "新規",
      rename: "名前",
      del: "削除",
      delConfirm: "このスレッドを削除しますか？",
      settings: "設定",
      presets: "プリセット",
      chat: "チャット",
      coder: "コーダー",
      observer: "オブザーバー",
      proposals: "提案",
      tasks: "タスク",
      planningTasks: "タスク生成中…",
      sendToCoder: "Coderへ送る",
      sendToObserver: "Observerへ送る",
      done: "完了",
      approved: "承認済み",
      alreadyApproved: "すでに送信済み",
      phaseMismatch: "フェーズ不一致",
      phaseMismatchConfirm: "フェーズが一致しません（現在: {cur} / 提案: {prop}）。それでもCoderへ送りますか？",
      applyMeta: "適用",
      includeCoderContext: "Coder状況を付与",
      send: "送信",
      stop: "停止",
      exportMd: "Markdown",
      clear: "消去",
      provider: "プロバイダ",
      model: "モデル",
      chatModel: "チャットモデル",
      codeModel: "コードモデル",
      fetchModels: "モデル一覧取得",
      baseUrl: "Base URL",
      apiKey: "APIキー",
      apiKeyChat: "チャットAPIキー",
      apiKeyCode: "コードAPIキー",
      apiKeyObserver: "Observer APIキー",
      codeProvider: "コード用プロバイダ",
      codeBaseUrl: "コード用Base URL",
      observerProvider: "Observerプロバイダ",
      observerBaseUrl: "Observer Base URL",
      observerModel: "Observerモデル",
      sameAsChat: "（チャットと同じ）",
      mode: "モード",
      persona: "ペルソナ",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "ストリーミング",
      cot: "CoT",
      brief: "簡易",
      structured: "構造化",
      deep: "本格",
      autoObserve: "自動実況",
      observerHint: "下の入力欄にメッセージを送信するか、設定で「自動実況」をONにしてください。",
      forceAgent: "エージェント常時ON",
      toolRoot: "作業ルート",
      workdir: "作業ディレクトリ",
      findInThread: "スレッド内検索…",
      noMatches: "一致するメッセージがありません",
      on: "ON",
      off: "OFF",
      diff: "diff",
      diffHint: "diff批評モードで自動注入されます",
      placeholder: "ここに入力。Enterで送信 (Shift+Enterで改行)",
      ready: "ready",
      sending: "sending…",
      streaming: "streaming…",
      error: "error",
      fetchFailed: "通信できません（obstral serve が起動してる？ポート合ってる？）",
      copy: "コピー",
      reader: "読む",
      refresh: "更新",
      keys: "APIキー状況",
      loopDetected: "ループ検出",
      observerIntensity: "Observer強度",
      polite: "丁寧",
      critical: "批評",
      brutal: "容赦なし",
      vibeAgent: "Vibe agent",
      vibeMaxTurns: "Vibe max turns",
      details: "詳細",
      hide: "隠す",
      serverOutdated: "サーバ古い",
      more: "もっと見る",
      less: "折りたたむ",
      shortcuts: "ショートカット",
      close: "閉じる",
      delQ: "削除しますか？",
      yes: "はい",
      no: "いいえ",
      sendMsg: "送信",
      newline: "改行",
      toggleHelp: "ヘルプ表示切替",
      closeModals: "モーダルを閉じる / リネーム取消",
      stopStreamingCoder: "ストリーミング停止（Coder/Observer）",
      insertCliTemplate: "CLIテンプレート",
      editApproval: "編集承認",
      commandApproval: "コマンド承認",
      pendingEdits: "保留中の編集",
      approve: "承認",
      reject: "却下",
    },
    fr: {
      threads: "Fils",
      newThread: "Nouveau",
      rename: "Renommer",
      del: "Suppr.",
      delConfirm: "Supprimer ce fil ?",
      settings: "Réglages",
      presets: "Préréglages",
      chat: "Chat",
      coder: "Codeur",
      observer: "Observateur",
      proposals: "Propositions",
      tasks: "Tâches",
      planningTasks: "Planification des tâches…",
      sendToCoder: "Envoyer au codeur",
      sendToObserver: "Envoyer à l'observer",
      done: "Fait",
      approved: "approuvé",
      alreadyApproved: "Déjà approuvé",
      phaseMismatch: "Phase incompatible",
      phaseMismatchConfirm: "Phase incompatible (actuelle: {cur} / proposition: {prop}). Envoyer quand même ?",
      applyMeta: "Appliquer",
      includeCoderContext: "Inclure contexte codeur",
      insertCliTemplate: "Template CLI",
      editApproval: "Approbation édition",
      commandApproval: "Approbation commande",
      toolRoot: "Racine outils",
      workdir: "Répertoire de travail",
      findInThread: "Rechercher dans le fil…",
      noMatches: "Aucun résultat",
      pendingEdits: "Éditions en attente",
      approve: "Approuver",
      reject: "Rejeter",
      send: "Envoyer",
      stop: "Stop",
      exportMd: "Markdown",
      clear: "Effacer",
      provider: "Fournisseur",
      model: "Modèle",
      chatModel: "Modèle chat",
      codeModel: "Modèle code",
      fetchModels: "Charger modèles",
      baseUrl: "Base URL",
      codeProvider: "Provider code",
      codeBaseUrl: "Base URL code",
      observerProvider: "Provider observer",
      observerBaseUrl: "Base URL observer",
      observerModel: "Modèle observer",
      apiKey: "Clé API",
      apiKeyChat: "Clé API chat",
      apiKeyCode: "Clé API code",
      apiKeyObserver: "Clé API observer",
      sameAsChat: "(idem chat)",
      mode: "Mode",
      persona: "Persona",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "Streaming",
      cot: "CoT",
      brief: "Bref",
      structured: "Structuré",
      deep: "Approfondi",
      on: "ON",
      off: "OFF",
      diff: "diff",
      diffHint: "Auto-injecté en mode revue de diff",
      placeholder: "Tapez ici. Entrée pour envoyer (Maj+Entrée pour nouvelle ligne)",
      ready: "prêt",
      sending: "envoi…",
      streaming: "stream…",
      error: "erreur",
      fetchFailed: "Échec de requête (OBSTRAL serve est-il lancé ?)",
      copy: "Copier",
      reader: "Lire",
      refresh: "Rafraîchir",
      keys: "Clés API",
      loopDetected: "Boucle détectée",
      observerIntensity: "Intensité observer",
      polite: "poli",
      critical: "critique",
      brutal: "brutal",
      vibeAgent: "Agent Vibe",
      vibeMaxTurns: "Tours max Vibe",
      details: "détails",
      hide: "masquer",
      autoObserve: "Auto-commenter",
      observerHint: "Tapez un message ci-dessous pour commencer, ou activez Auto-commenter dans les paramètres.",
      forceAgent: "Mode agent (toujours)",
      serverOutdated: "Serveur obsolète",
      more: "Afficher plus",
      less: "Réduire",
      shortcuts: "Raccourcis clavier",
      close: "Fermer",
      delQ: "Supprimer ?",
      yes: "Oui",
      no: "Non",
      sendMsg: "Envoyer",
      newline: "Nouvelle ligne",
      toggleHelp: "Afficher/masquer l'aide",
      closeModals: "Fermer les modales / annuler le renommage",
      stopStreamingCoder: "Arrêter le streaming (Coder/Observer)",
    },
  };

  function tr(lang, key) {
    return (I18N[lang] && I18N[lang][key]) || I18N.en[key] || key;
  }

  // ── Plugin Registry ───────────────────────────────────────────────────────────
  // Extend OBSTRAL without forking the source.
  //
  // Usage — load your plugin via <script src="my-plugin.js"></script> before app.js:
  //   registerObserverPlugin({ name, onProposal, onHealth, onPhase })
  //   registerPhase(key, { label, color, description })
  //   registerValidator(fn)   // fn(proposals[]) -> proposals[]
  //
  // Hooks are currently scaffolded (no-op). Integration points will be added
  // in parseProposals() and parseHealthScore() as Phase C matures.
  const _OBSTRAL_PLUGINS = { observer: [], phases: {}, validators: [] };
  function registerObserverPlugin(p) { _OBSTRAL_PLUGINS.observer.push(p); }
  function registerPhase(key, cfg)   { _OBSTRAL_PLUGINS.phases[key] = cfg; }
  function registerValidator(fn)     { _OBSTRAL_PLUGINS.validators.push(fn); }

  // ╔══════════════════════════════════════════════════════════╗
  // ║  SECTION: Utils                                          ║
  // ╚══════════════════════════════════════════════════════════╝
  function safeJsonParse(s, fallback) {
    try {
      return JSON.parse(s);
    } catch (_) {
      return fallback;
    }
  }

  function uid() {
    try {
      if (crypto && crypto.randomUUID) return crypto.randomUUID();
    } catch (_) {}
    return "id-" + Math.random().toString(16).slice(2) + "-" + Date.now().toString(16);
  }

  function extractJsonObject(text) {
    const s = String(text || "").trim();
    if (!s) return null;
    const i = s.indexOf("{");
    const j = s.lastIndexOf("}");
    if (i < 0 || j <= i) return null;
    return s.slice(i, j + 1);
  }

  function parseTasksJson(text) {
    const raw = extractJsonObject(text);
    if (!raw) return [];
    const j = safeJsonParse(raw, null);
    if (!j || typeof j !== "object" || !Array.isArray(j.tasks)) return [];
    return j.tasks
      .filter((t) => t && typeof t === "object")
      .map((t) => ({
        id: typeof t.id === "string" && t.id ? t.id : uid(),
        target: String(t.target || "").trim().toLowerCase() === "observer" ? "observer" : "coder",
        title: String(t.title || "").trim(),
        body: String(t.body || "").trim(),
        phase: String(t.phase || "any").trim().toLowerCase(),
        priority: typeof t.priority === "number" ? Math.max(0, Math.min(100, t.priority)) : null,
        status: String(t.status || "new").trim().toLowerCase() || "new",
        createdAt: typeof t.createdAt === "number" ? t.createdAt : Date.now(),
      }))
      .filter((t) => t.title || t.body);
  }

  // Similarity + proposal parsing helpers live in `web/observer/logic.js`
  // and are exposed on `window.OBSTRAL.observer`.

  function autoObservePrompt(uiLang) {
    const l = String(uiLang || "").trim().toLowerCase();
    if (l === "fr") {
      return "[AUTO-OBSERVE] Le Coder vient de produire une nouvelle sortie. Fais une critique en mode commentaire live.\n"
        + "1) Commence par UNE phrase: qu'est-ce qui vient d'arriver.\n"
        + "2) Analyse les risques sur 5 axes: exactitude, sécurité, fiabilité, performance, maintenabilité.\n"
        + "3) Termine par LES QUATRE blocs structurés dans cet ordre exact:\n"
        + "   --- phase ---          (core | feature | polish)\n"
        + "   --- proposals ---      (scorées, avec quote pour warn/crit)\n"
        + "   --- critical_path ---\n"
        + "   --- health ---         (score: N  rationale: une phrase)";
    }
    if (l === "en") {
      return "[AUTO-OBSERVE] The Coder produced new output. Commentate and critique live.\n"
        + "1) Start with ONE sentence: what just happened.\n"
        + "2) Risk scan across 5 axes: correctness, security, reliability, performance, maintainability.\n"
        + "3) End with ALL FOUR structured blocks in this exact order:\n"
        + "   --- phase ---          (core | feature | polish)\n"
        + "   --- proposals ---      (scored, with quote for warn/crit)\n"
        + "   --- critical_path ---\n"
        + "   --- health ---         (score: N  rationale: one sentence)";
    }
    return "[AUTO-OBSERVE] コーダーが新しいアウトプットを生成した。実況しながら批評せよ。\n"
      + "1) まず一文で「何が起きたか」を述べる。\n"
      + "2) 5軸（正しさ/セキュリティ/信頼性/性能/保守性）でリスクを洗い出す。\n"
      + "3) 最後に以下の4ブロックを必ず順番通りに出力する:\n"
      + "   --- phase ---          (core | feature | polish)\n"
      + "   --- proposals ---      (スコア付き、warn/crit には quote 必須)\n"
      + "   --- critical_path ---\n"
      + "   --- health ---         (score: N  rationale: 一文)";
  }

  const PROVIDERS = [
    { k: "mistral", l: { ja: "Mistral", en: "Mistral", fr: "Mistral" } },
    { k: "codestral", l: { ja: "Codestral", en: "Codestral", fr: "Codestral" } },
    { k: "mistral-cli", l: { ja: "Mistral CLI", en: "Mistral CLI", fr: "Mistral CLI" } },
    { k: "openai-compatible", l: { ja: "OpenAI互換", en: "OpenAI compat", fr: "Compat OpenAI" } },
    { k: "gemini", l: { ja: "Gemini", en: "Gemini", fr: "Gemini" } },
    { k: "anthropic", l: { ja: "Anthropic", en: "Anthropic", fr: "Anthropic" } },
    { k: "hf", l: { ja: "HF local", en: "HF local", fr: "HF local" } },
  ];

  const MODES = [
    { k: "VIBE", l: { ja: "VIBE", en: "VIBE", fr: "VIBE" } },
    { k: "壁打ち", l: { ja: "壁打ち", en: "Ideation", fr: "Idéation" } },
    { k: "実況", l: { ja: "実況", en: "Live", fr: "Direct" } },
    { k: "Observer", l: { ja: "Observer", en: "Observer", fr: "Observateur" } },
    { k: "diff批評", l: { ja: "diff批評", en: "Diff review", fr: "Revue diff" } },
  ];

  const CODER_MODES = MODES.filter((m) => m && m.k !== "Observer");
  const OBSERVER_MODES = MODES.filter((m) => m && m.k === "Observer");

  const PERSONAS = ["default", "novelist", "cynical", "cheerful", "thoughtful", "sensei", "duck"];
  const CHAT_PERSONAS = [
    { key: "cheerful",   icon: "😊", ja: "陽気",   en: "Cheerful",    fr: "Enjoué" },
    { key: "thoughtful", icon: "🤔", ja: "思慮深い", en: "Thoughtful",  fr: "Réfléchi" },
    { key: "sensei",     icon: "🧙", ja: "師匠",    en: "Sensei",      fr: "Sensei" },
    { key: "cynical",    icon: "😏", ja: "皮肉屋",  en: "Cynical",     fr: "Cynique" },
    { key: "duck",       icon: "🦆", ja: "ゴム鴨",  en: "Duck",        fr: "Canard" },
  ];

  const PRESETS = {
    vibe: {
      vibe: true,
      provider: "mistral",
      model: "codestral-latest",
      chatModel: "mistral-small-latest",
      codeModel: "codestral-latest",
      baseUrl: "https://api.mistral.ai/v1",
      codeProvider: "codestral",
      codeBaseUrl: "https://codestral.mistral.ai/v1",
      mode: "VIBE",
    },
    openai: {
      vibe: false,
      provider: "openai-compatible",
      model: "gpt-4o-mini",
      chatModel: "gpt-4o-mini",
      codeModel: "gpt-4o-mini",
      baseUrl: "https://api.openai.com/v1",
      mode: "壁打ち",
    },
    gemini: {
      vibe: false,
      provider: "gemini",
      model: "gemini-2.0-flash",
      chatModel: "gemini-2.0-flash",
      codeModel: "gemini-2.0-flash",
      baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
      mode: "壁打ち",
    },
    codestral: {
      vibe: true,
      provider: "codestral",
      model: "codestral-latest",
      chatModel: "mistral-small-latest",
      codeModel: "codestral-latest",
      baseUrl: "https://codestral.mistral.ai/v1",
      mode: "VIBE",
    },
    anthropic: {
      vibe: false,
      provider: "anthropic",
      model: "claude-3-5-sonnet-latest",
      chatModel: "claude-3-5-sonnet-latest",
      codeModel: "claude-3-5-sonnet-latest",
      baseUrl: "https://api.anthropic.com/v1",
      mode: "壁打ち",
    },
  };

  const DEFAULT_CONFIG = {
    ...PRESETS.openai,
    cot: "brief",
    autonomy: "longrun",
    requireEditApproval: true,
    requireCommandApproval: true,
    autoObserve: false,
    forceAgent: true,
    toolRoot: ".tmp",
    mistralCliAgent: "",
    mistralCliMaxTurns: "",
    codeProvider: "",
    codeBaseUrl: "",
    observerProvider: "",
    observerBaseUrl: "",
    observerModel: "",
    persona: "default",
    chatPersona: "cheerful",
    observerMode: "Observer",
    observerPersona: "novelist",
    observerIntensity: "critical",
    includeCoderContext: true,
    temperature: "0.7",
    maxTokens: "1024",
    timeoutSeconds: "120",
    stream: true,
  };

  function numOrUndef(v) {
    const s = String(v == null ? "" : v).trim();
    if (!s) return undefined;
    const n = Number(s);
    return Number.isFinite(n) ? n : undefined;
  }

  function strOrUndef(v) {
    const s = String(v == null ? "" : v).trim();
    return s ? s : undefined;
  }

  function buildReq(cfg, apiKey, history, input, diff) {
    return {
      input,
      history: history && history.length ? history : undefined,
      diff: strOrUndef(diff),
      vibe: cfg.vibe ? true : undefined,
      provider: strOrUndef(cfg.provider),
      model: strOrUndef(cfg.model),
      chat_model: strOrUndef(cfg.chatModel || cfg.model),
      code_model: strOrUndef(cfg.codeModel || cfg.model),
      base_url: strOrUndef(cfg.baseUrl),
      api_key: strOrUndef(apiKey),
      tool_root: strOrUndef(cfg.toolRoot),
      mistral_cli_agent: strOrUndef(cfg.mistralCliAgent),
      mistral_cli_max_turns: numOrUndef(cfg.mistralCliMaxTurns),
      mode: strOrUndef(cfg.mode),
      cot: strOrUndef(cfg.cot),
      autonomy: strOrUndef(cfg.autonomy),
      require_edit_approval: typeof cfg.requireEditApproval === "boolean" ? cfg.requireEditApproval : undefined,
      require_command_approval: typeof cfg.requireCommandApproval === "boolean" ? cfg.requireCommandApproval : undefined,
      persona: strOrUndef(cfg.persona),
      temperature: numOrUndef(cfg.temperature),
      max_tokens: numOrUndef(cfg.maxTokens),
      timeout_seconds: numOrUndef(cfg.timeoutSeconds),
    };
  }

  function avatarLabel(m) {
    if (m && m.role === "user") return "U";
    if (m && m.pane === "chat") return "A";
    return m && m.pane === "observer" ? "O" : "C";
  }

  function whoLabel(m, lang) {
    const L = String(lang || "ja").trim().toLowerCase();
    if (m && m.role === "user") return L === "fr" ? "Vous" : L === "en" ? "You" : "あなた";
    if (m && m.pane === "chat") return "AI";
    return tr(L, m && m.pane === "observer" ? "observer" : "coder");
  }

  function makeThread(title) {
    return {
      id: uid(),
      title: title || "Untitled",
      createdAt: Date.now(),
      updatedAt: Date.now(),
      workdir: "",
      messages: [],
      tasks: [],
    };
  }

  function titleFrom(text) {
    const t = String(text || "").trim().replace(/\\s+/g, " ");
    if (!t) return "Untitled";
    return t.length > 28 ? t.slice(0, 28) + "…" : t;
  }

  function downloadText(filename, text) {
    const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1500);
  }

  function transcriptMd(thread, meta) {
    const lines = ["# OBSTRAL transcript", ""];
    if (meta) {
      lines.push("```");
      Object.keys(meta).forEach((k) => lines.push(`${k}: ${meta[k]}`));
      lines.push("```", "");
    }
    (thread.messages || []).forEach((m) => {
      const pane = m.pane === "observer" ? "observer" : "coder";
      lines.push(`## ${pane} / ${m.role}`, "", String(m.content || "").trimEnd(), "");
    });
    return lines.join("\\n");
  }

  async function readFileText(file) {
    return new Promise((resolve, reject) => {
      const r = new FileReader();
      r.onerror = () => reject(new Error("read failed"));
      r.onload = () => resolve(String(r.result || ""));
      r.readAsText(file);
    });
  }

  // ╔══════════════════════════════════════════════════════════╗
  // ║  SECTION: API & Streaming                                ║
  // ╚══════════════════════════════════════════════════════════╝
  async function postJson(url, body, signal) {
    const resp = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal,
    });

    const ct = (resp.headers.get("content-type") || "").toLowerCase();
    if (!resp.ok) {
      if (ct.includes("application/json")) {
        const j = await resp.json().catch(() => ({}));
        throw new Error(j.error || `HTTP ${resp.status}`);
      }
      const t = await resp.text().catch(() => "");
      throw new Error(t.trim() || `HTTP ${resp.status}`);
    }

    if (ct.includes("application/json")) return resp.json();
    return resp.text();
  }

  function parseSseFrame(frameText) {
    const lines = frameText.split(/\r?\n/);
    let event = "message";
    const dataLines = [];
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (!line) continue;
      if (line.startsWith(":")) continue;
      if (line.startsWith("event:")) event = line.slice(6).trim();
      if (line.startsWith("data:")) dataLines.push(line.slice(5).trimStart());
    }
    return { event, data: dataLines.join("\n") };
  }

  async function streamChat(body, onEvent, signal) {
    const resp = await fetch("/api/chat_stream", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal,
    });

    const ct = (resp.headers.get("content-type") || "").toLowerCase();
    if (!resp.ok) {
      if (ct.includes("application/json")) {
        const j = await resp.json().catch(() => ({}));
        throw new Error(j.error || `HTTP ${resp.status}`);
      }
      throw new Error(`HTTP ${resp.status}`);
    }

    if (!resp.body || !resp.body.getReader) throw new Error("stream not supported");

    const reader = resp.body.getReader();
    const dec = new TextDecoder();
    let buf = "";

    while (true) {
      const r = await reader.read();
      if (r.done) break;
      buf += dec.decode(r.value || new Uint8Array(), { stream: true });

      while (true) {
        let idx = buf.indexOf("\n\n");
        let sep = 2;
        const idx2 = buf.indexOf("\r\n\r\n");
        if (idx2 !== -1 && (idx === -1 || idx2 < idx)) {
          idx = idx2;
          sep = 4;
        }
        if (idx === -1) break;

        const frame = buf.slice(0, idx);
        buf = buf.slice(idx + sep);
        onEvent(parseSseFrame(frame));
      }
    }
  }

  // ── Provider colors ───────────────────────────────────────────────────────────
  const PROVIDER_COLORS = {
    "mistral":           "#2dd4bf",
    "codestral":         "#14b8a6",
    "mistral-cli":       "#34d399",
    "openai-compatible": "#60a5fa",
    "gemini":            "#a3e635",
    "anthropic":         "#fb7185",
    "hf":                "#fbbf24",
  };

  // ╔══════════════════════════════════════════════════════════╗
  // ║  SECTION: Markdown Renderer                              ║
  // ╚══════════════════════════════════════════════════════════╝
  function renderInlineMd(text, baseKey) {
    const parts = [];
    const re = /(\*\*(.+?)\*\*|`([^`]+)`)/g;
    let last = 0, m, k = 0;
    while ((m = re.exec(text)) !== null) {
      if (m.index > last) parts.push(text.slice(last, m.index));
      if (m[0].startsWith("**")) {
        parts.push(e("strong", { key: baseKey + k++ }, m[2]));
      } else {
        parts.push(e("code", {
          key: baseKey + k++,
          style: { fontFamily: "var(--mono)", fontSize: "11px", background: "rgba(255,255,255,0.1)", padding: "1px 5px", borderRadius: 4 },
        }, m[3]));
      }
      last = m.index + m[0].length;
    }
    if (last < text.length) parts.push(text.slice(last));
    return parts;
  }

  const SANDBOX = (window.OBSTRAL && window.OBSTRAL.sandbox) ? window.OBSTRAL.sandbox : {};
  const normalizePathSep = SANDBOX.normalizePathSep;
  const safeThreadId = SANDBOX.safeThreadId;
  const safeWorkdir = SANDBOX.safeWorkdir;
  const resolvedThreadRoot = SANDBOX.resolvedThreadRoot;
  const resolvedCwd = SANDBOX.resolvedCwd;
  if (!normalizePathSep || !safeThreadId || !safeWorkdir || !resolvedThreadRoot || !resolvedCwd) {
    throw new Error("OBSTRAL UI: missing sandbox helpers (core/sandbox.js not loaded)");
  }

  const OBSERVER = (window.OBSTRAL && window.OBSTRAL.observer) ? window.OBSTRAL.observer : {};
  const normalizeForSim = OBSERVER.normalizeForSim;
  const tokenSetForSim = OBSERVER.tokenSetForSim;
  const jaccardSim = OBSERVER.jaccardSim;
  const similarity = OBSERVER.similarity;
  const parseProposals = OBSERVER.parseProposals;
  const parseCriticalPath = OBSERVER.parseCriticalPath;
  const parseHealthScore = OBSERVER.parseHealthScore;
  const stripObserverMeta = OBSERVER.stripObserverMeta;
  if (
    !normalizeForSim
    || !tokenSetForSim
    || !jaccardSim
    || !similarity
    || !parseProposals
    || !parseCriticalPath
    || !parseHealthScore
    || !stripObserverMeta
  ) {
    throw new Error("OBSTRAL UI: missing observer helpers (observer/logic.js not loaded)");
  }

  const EXEC = (window.OBSTRAL && window.OBSTRAL.exec) ? window.OBSTRAL.exec : {};
  const isWindowsHost = EXEC.isWindowsHost;
  const stripShellTranscript = EXEC.stripShellTranscript;
  const dangerousCommandReason = EXEC.dangerousCommandReason;
  const gitRepoHint = EXEC.gitRepoHint;
  const normalizeExecScript = EXEC.normalizeExecScript;
  if (
    !isWindowsHost
    || !stripShellTranscript
    || !dangerousCommandReason
    || !gitRepoHint
    || !normalizeExecScript
  ) {
    throw new Error("OBSTRAL UI: missing exec helpers (core/exec.js not loaded)");
  }

  function confirmDangerous(uiLang, reason) {
    const l = String(uiLang || "").trim().toLowerCase();
    const r = String(reason || "").trim();
    const msg =
      l === "fr"
        ? `Commande dangereuse détectée: ${r}\nExécuter quand même ?`
        : l === "en"
          ? `Dangerous command detected: ${r}\nRun anyway?`
          : `危険なコマンドを検出: ${r}\nそれでも実行しますか？`;
    try { return window.confirm(msg); } catch (_) { return false; }
  }

  function confirmPhaseMismatch(uiLang, curPhase, propPhase) {
    const cur = String(curPhase || "").trim() || "n/a";
    const prop = String(propPhase || "").trim() || "n/a";
    const tmpl = tr(uiLang, "phaseMismatchConfirm");
    const msg = String(tmpl || "")
      .replace(/\{cur\}/g, cur)
      .replace(/\{prop\}/g, prop);
    try { return window.confirm(msg); } catch (_) { return false; }
  }

  // (exec helpers moved to web/core/exec.js)

  // Render a unified diff block with per-line colours.
  function renderDiffBody(codeText) {
    const colorOf = (line) => {
      if (/^diff\s/.test(line) || /^index\s/.test(line) || /^new file/.test(line) || /^deleted file/.test(line))
        return "rgba(96,165,250,0.9)";   // blue  — file header
      if (/^(\+\+\+|---)/.test(line))
        return "rgba(255,255,255,0.85)"; // white — path line
      if (/^@@/.test(line))
        return "rgba(45,212,191,0.9)";  // cyan  — hunk header
      if (line.startsWith("+"))
        return "rgba(74,222,128,0.9)";  // green — addition
      if (line.startsWith("-"))
        return "rgba(251,113,133,0.9)"; // red   — deletion
      return "rgba(255,255,255,0.58)";  // faint — context
    };
    return e(
      "div",
      { style: { padding: "10px", overflow: "auto", maxHeight: 480 } },
      codeText.split("\n").map((line, i) =>
        e("div", {
          key: i,
          style: {
            color: colorOf(line),
            fontFamily: "var(--mono)",
            fontSize: 12,
            lineHeight: 1.45,
            whiteSpace: "pre",
            minHeight: "1em",
          },
        }, line || "")
      )
    );
  }

  const PRISM_LANG_MAP = {
    js: "javascript", jsx: "javascript",
    ts: "typescript", tsx: "typescript",
    py: "python",
    sh: "bash", shell: "bash", zsh: "bash", console: "bash",
    ps1: "powershell", pwsh: "powershell", ps: "powershell",
    rs: "rust",
    html: "markup", xml: "markup",
  };

  function hlCode(code, lang) {
    const norm = PRISM_LANG_MAP[lang.toLowerCase()] || lang.toLowerCase();
    const P = window.Prism;
    if (!P) {
      return code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }
    const grammar = P.languages[norm];
    if (!grammar) {
      return code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }
    try { return P.highlight(code, grammar, norm); } catch (_) {
      return code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }
  }

  function parseMarkdown(text, execRes, onRun, onOpen) {
    const lines = text.split("\n");
    const out = [];
    let i = 0, k = 0;
    while (i < lines.length) {
      const line = lines[i];
      if (line.startsWith("```")) {
        const rawLang = line.slice(3).trim();
        // Detect optional filepath appended to lang: e.g. "python src/main.py"
        const langParts = rawLang.split(/\s+/);
        const lang = langParts[0];
        const filePath = (onOpen && langParts.length >= 2 &&
          (langParts[1].includes('/') || langParts[1].includes('\\') || langParts[1].includes('.')))
          ? langParts.slice(1).join(' ') : null;
        const code = [];
        i++;
        while (i < lines.length && !lines[i].startsWith("```")) { code.push(lines[i]); i++; }
        const codeText = code.join("\n");
        const blockIdx = k;
        const win = isWindowsHost();
        const isRunnable = onRun && (win
          ? /^(bash|sh|shell|zsh|console|powershell|pwsh|ps1|ps)$/i.test(lang)
          : /^(bash|sh|shell|zsh|console)$/i.test(lang)
        );
        const res = execRes && execRes[blockIdx];
        const isDiff = /^(diff|patch)$/i.test(lang);
        out.push(e("div", { className: "code", key: k++ },
          e("div", { className: "code-head" },
            e("span", null, filePath ? (lang + " " + filePath) : (lang || "code")),
            isRunnable && e("button", {
              className: "btn-run",
              disabled: !!(res && res.running),
              onClick: () => onRun(blockIdx, lang, codeText),
            }, res && res.running ? "⏳" : "▶ run"),
            filePath && e("button", {
              className: "btn-open",
              title: filePath,
              onClick: () => onOpen(filePath),
            }, "📂 open"),
            e("button", {
              style: { marginLeft: (isRunnable || filePath) ? "0" : "auto", background: "none", border: "none", color: "var(--muted)", cursor: "pointer", fontSize: "11px" },
              onClick: () => navigator.clipboard && navigator.clipboard.writeText(codeText).catch(() => {}),
            }, "⎘ copy"),
          ),
          isDiff ? renderDiffBody(codeText) : e("pre", {
            className: "language-" + (PRISM_LANG_MAP[lang.toLowerCase()] || lang.toLowerCase()),
            dangerouslySetInnerHTML: { __html: hlCode(codeText, lang) },
          }),
          res && !res.running && e("div", {
              className: "exec-result " + (res.exit_code === 0 ? "exec-ok" : "exec-err"),
            },
            res.stdout && e("pre", null, res.stdout),
            res.stderr && e("pre", { style: { color: "var(--warn)" } }, res.stderr),
            e("div", { className: "exec-meta" }, "exit " + res.exit_code),
          ),
        ));
        i++; continue;
      }
      const hm = line.match(/^(#{1,3})\s+(.+)/);
      if (hm) {
        const sz = hm[1].length === 1 ? "15px" : hm[1].length === 2 ? "14px" : "13px";
        out.push(e("div", { key: k++, style: { fontWeight: 700, fontSize: sz, margin: "10px 0 4px" } }, hm[2]));
        i++; continue;
      }
      if (/^---+$/.test(line.trim())) {
        out.push(e("div", { key: k++, className: "hr" }));
        i++; continue;
      }
      const lm = line.match(/^(\s*[-*]|\s*\d+\.)\s+(.+)/);
      if (lm) {
        out.push(e("div", { key: k++, style: { display: "flex", gap: 6, marginBottom: 2 } },
          e("span", { style: { color: "var(--faint)", flexShrink: 0 } }, "·"),
          e("span", null, renderInlineMd(lm[2], k)),
        ));
        i++; continue;
      }
      if (line.trim() === "") {
        out.push(e("div", { key: k++, style: { height: 6 } }));
        i++; continue;
      }
      out.push(e("div", { key: k++, style: { marginBottom: 2 } }, renderInlineMd(line, k)));
      i++;
    }
    return out;
  }

  function estimateTokens(text) {
    return Math.max(1, Math.ceil(String(text || "").length / 4));
  }

  function relativeTime(ts, uiLang) {
    const l = String(uiLang || "").trim().toLowerCase();
    const d = Date.now() - (ts || 0);
    if (d < 60000) return l === "fr" ? "à l'instant" : (l === "en" ? "just now" : "たった今");
    if (d < 3600000) {
      const m = Math.max(1, Math.floor(d / 60000));
      return l === "fr" ? `il y a ${m}m` : (l === "en" ? `${m}m ago` : `${m}分前`);
    }
    if (d < 86400000) {
      const h = Math.max(1, Math.floor(d / 3600000));
      return l === "fr" ? `il y a ${h}h` : (l === "en" ? `${h}h ago` : `${h}時間前`);
    }
    const days = Math.max(1, Math.floor(d / 86400000));
    return l === "fr" ? `il y a ${days}j` : (l === "en" ? `${days}d ago` : `${days}日前`);
  }

  // ── UI Render helpers ─────────────────────────────────────────────────────────

  // Renders message content with <think>…</think> blocks dimmed separately.
  function renderWithThink(text, execRes, onRun, onOpen) {
    const re = /<think>([\s\S]*?)<\/think>/gi;
    const parts = [];
    let last = 0;
    let k = 0;
    let m;
    while ((m = re.exec(text)) !== null) {
      if (m.index > last) {
        parts.push(...parseMarkdown(text.slice(last, m.index), execRes, onRun, onOpen));
      }
      parts.push(e("div", { key: "think" + k++, className: "think-block" }, m[1].trim()));
      last = m.index + m[0].length;
    }
    if (last < text.length) {
      parts.push(...parseMarkdown(text.slice(last), execRes, onRun, onOpen));
    }
    return parts;
  }

  // Extract a short sequential list like "1) ...\n2) ..." as clickable choices.
  // Used to support "multi-choice clarification" flows from VIBE / LLMs.
  function extractChoices(text) {
    const s = String(text || "");
    if (!s) return [];
    const lines = s.split("\n");
    const cueRe = /(choose|pick|select|which|option|choice|選ん|どれ|どちら|択|choisissez|s[ée]lectionnez)/i;
    const itemRe = /^\s*(\d{1,2})[).:\-]\s+(.+)\s*$/;

    for (let i = 0; i < lines.length; i++) {
      const m = itemRe.exec(lines[i]);
      if (!m) continue;
      const n0 = Number(m[1]);
      if (n0 !== 1) continue;
      const ctx = lines.slice(Math.max(0, i - 4), i + 1).join("\n");
      if (!cueRe.test(ctx)) continue;

      const out = [];
      let expected = 1;
      for (let j = i; j < lines.length; j++) {
        const mj = itemRe.exec(lines[j]);
        if (!mj) break;
        const nj = Number(mj[1]);
        const tj = String(mj[2] || "").trim();
        if (!Number.isFinite(nj) || nj !== expected || !tj) break;
        out.push({ n: nj, text: tj });
        expected++;
        if (out.length >= 6) break;
      }
      return out.length >= 2 ? out : [];
    }
    return [];
  }

  function choiceReply(uiLang, n, text) {
    const l = String(uiLang || "").trim().toLowerCase();
    const t = String(text || "").trim();
    if (l === "fr") return `Je choisis ${n}: ${t}`;
    if (l === "en") return `I choose ${n}: ${t}`;
    return `選択: ${n} ${t}`;
  }

  // ── Observer Logic ────────────────────────────────────────────────────────────
  // Observer parsing helpers are split into `web/observer/logic.js` (window.OBSTRAL.observer).

  function parseMetaPromptOp(toCoderText) {
    const s = String(toCoderText || "");
    const re = /^\s*META_(SET|APPEND)_(CODER|OBSERVER)\s*:\s*(.*)\s*$/im;
    const m = re.exec(s);
    if (!m) return null;
    const op = String(m[1] || "").toLowerCase() === "append" ? "append" : "set";
    const target = String(m[2] || "").toLowerCase() === "observer" ? "observer" : "coder";
    const head = String(m[3] || "").trim();
    // Everything after the directive line is considered part of the meta prompt.
    const rest = s.slice(m.index + m[0].length).replace(/^\r?\n/, "").trim();
    const text = (head && rest) ? (head + "\n" + rest) : (head || rest);
    if (!String(text || "").trim()) return null;
    return { op, target, text };
  }

  function parsePhase(text) {
    const s = String(text || "");
    const m = /---\s*phase\s*---/i.exec(s);
    if (!m) return null;
    const tail = s.slice(m.index + m[0].length).trimStart();
    const word = tail.split(/[\r\n\s(]/)[0].toLowerCase();
    return word === "core" || word === "feature" || word === "polish" ? word : null;
  }

  // ╔══════════════════════════════════════════════════════════╗
  // ║  SECTION: App Shell (React state + event handlers)       ║
  // ╚══════════════════════════════════════════════════════════╝
  function App() {
    const [lang, setLang] = useState(() => {
      const v = (localStorage.getItem(LS.lang) || "").trim();
      return v === "ja" || v === "en" || v === "fr" ? v : "ja";
    });
    const [status, setStatus] = useState(null);

    const [config, setConfig] = useState(() => {
      const v = safeJsonParse(localStorage.getItem(LS.config) || "null", null);
      if (!v || typeof v !== "object") return { ...DEFAULT_CONFIG };
      const cfg = { ...DEFAULT_CONFIG, ...v };
      if (!cfg.chatModel && cfg.model) cfg.chatModel = cfg.model;
      if (!cfg.codeModel && cfg.model) cfg.codeModel = cfg.model;
      if (!cfg.model && (cfg.chatModel || cfg.codeModel)) cfg.model = cfg.chatModel || cfg.codeModel;
      // Coder pane should never run in Observer mode (it disables autonomy/tooling and looks "broken").
      if (String(cfg.mode || "").trim() === "Observer") cfg.mode = DEFAULT_CONFIG.mode;
      if (!cfg.observerMode) cfg.observerMode = DEFAULT_CONFIG.observerMode;
      if (!cfg.chatPersona) cfg.chatPersona = DEFAULT_CONFIG.chatPersona;
      if (!cfg.observerPersona) cfg.observerPersona = DEFAULT_CONFIG.observerPersona;
      const oi0 = String(cfg.observerIntensity || "").trim().toLowerCase();
      if (oi0 === "polite" || oi0 === "critical" || oi0 === "brutal") cfg.observerIntensity = oi0;
      else cfg.observerIntensity = DEFAULT_CONFIG.observerIntensity;
      if (typeof cfg.includeCoderContext !== "boolean") cfg.includeCoderContext = !!DEFAULT_CONFIG.includeCoderContext;
      if (typeof cfg.requireEditApproval !== "boolean") cfg.requireEditApproval = !!DEFAULT_CONFIG.requireEditApproval;
      if (typeof cfg.requireCommandApproval !== "boolean") cfg.requireCommandApproval = !!DEFAULT_CONFIG.requireCommandApproval;
      if (typeof cfg.autoObserve !== "boolean") cfg.autoObserve = !!DEFAULT_CONFIG.autoObserve;
      if (typeof cfg.forceAgent !== "boolean") cfg.forceAgent = !!DEFAULT_CONFIG.forceAgent;
      if (typeof cfg.toolRoot !== "string") cfg.toolRoot = String(cfg.toolRoot || "");
      if (!String(cfg.toolRoot || "").trim()) cfg.toolRoot = DEFAULT_CONFIG.toolRoot;
      if (typeof cfg.mistralCliAgent !== "string") cfg.mistralCliAgent = String(cfg.mistralCliAgent || "");
      if (typeof cfg.mistralCliMaxTurns !== "string") cfg.mistralCliMaxTurns = String(cfg.mistralCliMaxTurns || "");
      if (typeof cfg.codeProvider !== "string") cfg.codeProvider = String(cfg.codeProvider || "");
      if (typeof cfg.codeBaseUrl !== "string") cfg.codeBaseUrl = String(cfg.codeBaseUrl || "");
      if (typeof cfg.observerProvider !== "string") cfg.observerProvider = String(cfg.observerProvider || "");
      if (typeof cfg.observerBaseUrl !== "string") cfg.observerBaseUrl = String(cfg.observerBaseUrl || "");
      if (typeof cfg.observerModel !== "string") cfg.observerModel = String(cfg.observerModel || "");
      const cot0 = String(cfg.cot || "").trim().toLowerCase();
      if (cot0 === "off") cfg.cot = "off";
      else if (cot0 === "structured") cfg.cot = "structured";
      else if (cot0 === "deep") cfg.cot = "deep";
      else cfg.cot = "brief";
      if (String(cfg.autonomy || "").trim().toLowerCase() === "off") cfg.autonomy = "off";
      else cfg.autonomy = "longrun";
      // One-time migration: switch legacy default Mistral config to OpenAI-compatible defaults.
      const provider0 = String(cfg.provider || "").trim();
      const baseUrl0 = String(cfg.baseUrl || "").trim().toLowerCase();
      if (provider0 === "mistral" && (!baseUrl0 || baseUrl0 === "https://api.mistral.ai/v1")) {
        cfg.provider = PRESETS.openai.provider;
        cfg.baseUrl = PRESETS.openai.baseUrl;
        cfg.model = PRESETS.openai.model;
        cfg.chatModel = PRESETS.openai.chatModel;
        cfg.codeModel = PRESETS.openai.codeModel;
        cfg.mode = PRESETS.openai.mode;
        cfg.vibe = false;
      }
      // Migrate legacy Mistral model ids that no longer exist.
      if (String(cfg.provider || "").trim() === "mistral") {
        const isDevstral2 = (x) => String(x || "").trim() === "devstral-2";
        if (isDevstral2(cfg.model) || isDevstral2(cfg.chatModel) || isDevstral2(cfg.codeModel)) {
          cfg.model = "codestral-latest";
          cfg.codeModel = "codestral-latest";
          if (!cfg.chatModel || isDevstral2(cfg.chatModel)) {
            cfg.chatModel = "mistral-small-latest";
          }
        }
      }
      return cfg;
    });
    const [chatApiKey, setChatApiKey] = useState("");
    const [codeApiKey, setCodeApiKey] = useState("");
    const [observerApiKey, setObserverApiKey] = useState("");
    const [diff, setDiff] = useState("");
    const [models, setModels] = useState([]);
    const [modelsLoading, setModelsLoading] = useState(false);
    const [modelsErr, setModelsErr] = useState("");
    const [modelsTarget, setModelsTarget] = useState("chat");
    const [pendingEdits, setPendingEdits] = useState([]);
    const [pendingBusy, setPendingBusy] = useState(false);
    const [metaBusy, setMetaBusy] = useState(false);
    const [observerSubTab, setObserverSubTab] = useState("analysis"); // "analysis" | "chat"
    const [chatInput, setChatInput] = useState("");
    const [sendingChat, setSendingChat] = useState(false);
    const [proposalModal, setProposalModal] = useState(null);
    const [proposalModalText, setProposalModalText] = useState("");
    const [readerModal, setReaderModal] = useState(null);
    const [planningTasks, setPlanningTasks] = useState(false);

    const [threadState, setThreadState] = useState(() => {
      let threads = safeJsonParse(localStorage.getItem(LS.threads) || "null", null);
      threads = Array.isArray(threads) ? threads : [];
      threads = threads
        .filter((t) => t && typeof t === "object" && typeof t.id === "string")
        .map((t) => ({
          id: t.id,
          title: typeof t.title === "string" && t.title.trim() ? t.title : "Untitled",
          createdAt: typeof t.createdAt === "number" ? t.createdAt : Date.now(),
          updatedAt: typeof t.updatedAt === "number" ? t.updatedAt : Date.now(),
          workdir: typeof t.workdir === "string" ? t.workdir : "",
          messages: Array.isArray(t.messages)
            ? t.messages
                .filter((m) => m && typeof m === "object")
                .map((m) => ({
                  id: typeof m.id === "string" && m.id ? m.id : uid(),
                  pane: m.pane === "observer" ? "observer" : m.pane === "chat" ? "chat" : "coder",
                  role: m.role === "assistant" ? "assistant" : "user",
                  content: typeof m.content === "string" ? m.content : String(m.content || ""),
                  ts: typeof m.ts === "number" ? m.ts : Date.now(),
                  streaming: !!m.streaming,
                }))
            : [],
          tasks: Array.isArray(t.tasks)
            ? t.tasks
                .filter((x) => x && typeof x === "object")
                .map((x) => ({
                  id: typeof x.id === "string" && x.id ? x.id : uid(),
                  target: String(x.target || "").trim().toLowerCase() === "observer" ? "observer" : "coder",
                  title: typeof x.title === "string" ? x.title : String(x.title || ""),
                  body: typeof x.body === "string" ? x.body : String(x.body || ""),
                  phase: typeof x.phase === "string" ? x.phase : "",
                  priority: typeof x.priority === "number" ? x.priority : null,
                  status: typeof x.status === "string" && x.status ? x.status : "new",
                  createdAt: typeof x.createdAt === "number" ? x.createdAt : Date.now(),
                }))
            : [],
        }));
      if (!threads.length) threads = [makeThread("Thread 1")];
      const active0 = (localStorage.getItem(LS.active) || "").trim();
      const active = threads.some((t) => t.id === active0) ? active0 : threads[0].id;
      return { threads, activeId: active };
    });

    const [coderInput, setCoderInput] = useState("");
    const [observerInput, setObserverInput] = useState("");
    const [coderFind, setCoderFind] = useState("");
    const [observerFind, setObserverFind] = useState("");
    const [sendingCoder, setSendingCoder] = useState(false);
    const [sendingObserver, setSendingObserver] = useState(false);
    const [loopInfo, setLoopInfo] = useState({ active: false, score: 0, depth: 0 });
    const [execResults, setExecResults] = useState({}); // { msgId: { blockIdx: {stdout,stderr,exit_code,running} } }
    const [expandedProposals, setExpandedProposals] = useState(new Set());
    const [expandedMsgs, setExpandedMsgs] = useState(new Set());
    const [copiedId, setCopiedId] = useState(null);
    const [toasts, setToasts] = useState([]);
    const [editingThreadId, setEditingThreadId] = useState(null);
    const [editingTitle, setEditingTitle] = useState("");
    const [confirmDeleteId, setConfirmDeleteId] = useState(null);
    const [showShortcuts, setShowShortcuts] = useState(false);
    const [splitPct, setSplitPct] = useState(() => {
      try {
        const v = Number(localStorage.getItem(LS.splitPct));
        if (Number.isFinite(v) && v >= 20 && v <= 80) return v;
      } catch (_) {}
      return 55;
    });
    const arenaRef = useRef(null);
    const isDraggingRef = useRef(false);

    const showToast = (msg, type = "info") => {
      const id = Date.now() + Math.random();
      setToasts((t) => [...t, { id, msg, type }]);
      setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), 3500);
    };

    const abortCoderRef = useRef(null);
    const abortObserverRef = useRef(null);
    const loopRef = useRef({ lastChecked: "", depth: 0 });
    const lastAutoObserveMsgRef = useRef(null);
    const saveTimer = useRef(null);
    const coderBodyRef = useRef(null);
    const observerBodyRef = useRef(null);
    const abortChatRef = useRef(null);
    const chatBodyRef = useRef(null);
    const toolRootInitRef = useRef(false);
    const ensuredWorkdirRef = useRef({});

    // Drag-to-resize pane handler.
    const onSplitDragStart = useCallback((e) => {
      e.preventDefault();
      isDraggingRef.current = true;
      const onMove = (ev) => {
        if (!isDraggingRef.current || !arenaRef.current) return;
        const rect = arenaRef.current.getBoundingClientRect();
        const x = ev.touches ? ev.touches[0].clientX : ev.clientX;
        const pct = Math.round(Math.min(80, Math.max(20, ((x - rect.left) / rect.width) * 100)));
        setSplitPct(pct);
      };
      const onUp = () => {
        isDraggingRef.current = false;
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
      };
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    }, []);

    // Global keyboard shortcuts.
    useEffect(() => {
      const onKey = (e) => {
        if (e.key === "?" && !["INPUT", "TEXTAREA", "SELECT"].includes(e.target.tagName)) {
          setShowShortcuts((v) => !v);
        }
        if ((e.ctrlKey || e.metaKey) && String(e.key || "").toLowerCase() === "k") {
          e.preventDefault();
          try { if (abortCoderRef.current) abortCoderRef.current.abort(); } catch (_) {}
          abortCoderRef.current = null;
          setSendingCoder(false);
          try { if (abortObserverRef.current) abortObserverRef.current.abort(); } catch (_) {}
          abortObserverRef.current = null;
          setSendingObserver(false);
        }
        if (e.key === "Escape") {
          setShowShortcuts(false);
          setProposalModal(null);
          setReaderModal(null);
          setEditingThreadId(null);
          setConfirmDeleteId(null);
        }
      };
      window.addEventListener("keydown", onKey);
      return () => window.removeEventListener("keydown", onKey);
    }, []);

    // Persist pane split ratio.
    useEffect(() => {
      try { localStorage.setItem(LS.splitPct, String(splitPct)); } catch (_) {}
    }, [splitPct]);

    const activeThread = useMemo(() => {
      return threadState.threads.find((t) => t.id === threadState.activeId) || threadState.threads[0];
    }, [threadState]);

    useEffect(() => {
      document.documentElement.lang = lang;
      localStorage.setItem(LS.lang, lang);
    }, [lang]);

    useEffect(() => {
      const safe = { ...config };
      delete safe.apiKey;
      localStorage.setItem(LS.config, JSON.stringify(safe));
    }, [config]);

    useEffect(() => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(() => {
        try {
          localStorage.setItem(LS.threads, JSON.stringify(threadState.threads));
          localStorage.setItem(LS.active, threadState.activeId);
        } catch (_) {}
      }, 350);
      return () => saveTimer.current && clearTimeout(saveTimer.current);
    }, [threadState]);

    useEffect(() => {
      loopRef.current = { lastChecked: "", depth: 0 };
      lastAutoObserveMsgRef.current = null;
      setLoopInfo({ active: false, score: 0, depth: 0 });
    }, [threadState.activeId]);

    useEffect(() => {
      // Detect repeated Observer replies (e.g. review -> rewrite -> review loops) and surface it.
      const msgs = paneMessages("observer");
      const asst = (msgs || []).filter((m) => m && m.role === "assistant" && !m.streaming && String(m.content || "").trim());
      if (asst.length < 2) return;

      const last = asst[asst.length - 1];
      if (!last || !last.id) return;
      if (loopRef.current.lastChecked === last.id) return;
      loopRef.current.lastChecked = last.id;

      const window = asst.slice(0, -1).slice(-4);
      let maxSim = 0;
      for (const prev of window) {
        maxSim = Math.max(maxSim, similarity(last.content, prev.content));
      }

      const minChars = 180;
      const detected = String(last.content || "").trim().length >= minChars && maxSim >= 0.80;
      let depth = loopRef.current.depth || 0;
      if (detected) depth = Math.min(18, depth + 1);
      else depth = Math.max(0, depth - 1);
      loopRef.current.depth = depth;

      setLoopInfo({ active: detected, score: maxSim, depth });
    }, [threadState]);

    // Auto-observe: fire Observer automatically when Coder finishes a response.
    useEffect(() => {
      if (!config.autoObserve) return;
      if (sendingObserver || sendingCoder) return;
      const coderMsgs = paneMessages("coder");
      // Find latest completed (non-streaming, non-empty) Coder assistant message.
      const lastAsst = [...coderMsgs].reverse().find(
        (m) => m.role === "assistant" && !m.streaming && String(m.content || "").trim().length > 40
      );
      if (!lastAsst) return;
      const key = (activeThread && activeThread.id ? activeThread.id : "") + ":" + lastAsst.id;
      if (lastAutoObserveMsgRef.current === key) return;
      lastAutoObserveMsgRef.current = key;
      // Slight delay to let React settle after streaming ends.
      const timer = setTimeout(() => {
        sendObserver(autoObservePrompt(lang));
      }, 700);
      return () => clearTimeout(timer);
    }, [threadState, config.autoObserve, sendingCoder, sendingObserver]);

    useEffect(() => {
      const d = Number(loopInfo.depth) || 0;
      const hue = Math.min(360, d * 20);
      if (typeof document === "undefined" || !document.body) return;
      if (d > 0) {
        document.body.classList.add("looping");
        document.body.style.setProperty("--loop-hue", hue + "deg");
        document.body.style.setProperty("--loop-sat", String(1 + Math.min(0.6, d * 0.05)));
      } else {
        document.body.classList.remove("looping");
        document.body.style.removeProperty("--loop-hue");
        document.body.style.removeProperty("--loop-sat");
      }
      return () => {
        if (typeof document === "undefined" || !document.body) return;
        document.body.classList.remove("looping");
        document.body.style.removeProperty("--loop-hue");
        document.body.style.removeProperty("--loop-sat");
      };
    }, [loopInfo.depth]);

    useEffect(() => {
      refreshStatus();
      refreshPendingEdits();
      const t = setInterval(() => refreshPendingEdits(), 3000);
      return () => clearInterval(t);
    }, []);

    // Safer default: run local commands under a scratch directory, not the OBSTRAL repo root.
    // This prevents nested git repos (embedded repo warnings) and accidental `git add .` fallout.
    useEffect(() => {
      if (toolRootInitRef.current) return;
      if (!status || !status.features || !status.features.exec) return;
      toolRootInitRef.current = true;

      const cur = String(config.toolRoot || "").trim();
      const desired = cur || ".tmp";
      if (!cur) setConfig((c) => ({ ...c, toolRoot: desired }));

      // Best-effort: ensure the default scratch dir exists so exec doesn't fail.
      if (desired === ".tmp") {
        const cmd = isWindowsHost()
          ? "New-Item -ItemType Directory -Force -Path '.tmp' | Out-Null"
          : "mkdir -p .tmp";
        postJson("/api/exec", { command: cmd }).catch(() => {});
      }
    }, [status && status.features && status.features.exec]);

    // When using the default scratch dir (.tmp), isolate each thread under .tmp/<threadId>.
    // This prevents collisions ("already exists") and the classic "embedded git repo" failure mode.
    useEffect(() => {
      if (!status || !status.features || !status.features.exec) return;
      if (!activeThread || !activeThread.id) return;
      const root0 = String(config.toolRoot || "").trim();
      const root = resolvedThreadRoot(root0, activeThread.id);
      const wd = resolvedCwd(root0, activeThread.id, activeThread.workdir);
      if (!root0 || !root) return;

      const ensureDir = (p) => {
        if (!p) return;
        const norm = normalizePathSep(p).replace(/\/+$/g, "");
        if (ensuredWorkdirRef.current[norm]) return;
        ensuredWorkdirRef.current[norm] = true;
        const cmd = isWindowsHost()
          ? ("New-Item -ItemType Directory -Force -Path " + psSingleQuote(p) + " | Out-Null")
          : ("mkdir -p " + JSON.stringify(p));
        postJson("/api/exec", { command: cmd }).catch(() => {});
      };

      // Always ensure the per-thread root exists (e.g. .tmp/<threadId>).
      ensureDir(root);
      // Also ensure the per-thread workdir exists, if set (e.g. .tmp/<threadId>/<repo>).
      if (wd && normalizePathSep(wd).replace(/\/+$/g, "") !== normalizePathSep(root).replace(/\/+$/g, "")) {
        ensureDir(wd);
      }
    }, [status && status.features && status.features.exec, config.toolRoot, activeThread && activeThread.id, activeThread && activeThread.workdir]);

    const refreshStatus = () => {
      fetch("/api/status")
        .then((r) => r.json())
        .then((j) => {
          try { window.__OBSTRAL_HOST_OS = j && j.host_os ? String(j.host_os) : ""; } catch (_) {}
          setStatus(j);
        })
        .catch(() => {});
    };

    const refreshPendingEdits = () => {
      fetch("/api/pending_edits")
        .then((r) => r.json())
        .then((j) => setPendingEdits(j && Array.isArray(j.pending) ? j.pending : []))
        .catch(() => {});
    };

    const resolvePendingEdit = async (id, approve) => {
      const eid = String(id || "").trim();
      if (!eid || pendingBusy) return;
      setPendingBusy(true);
      try {
        const resp = await postJson(approve ? "/api/approve_edit" : "/api/reject_edit", { id: eid });
        if (approve && resp && resp.item && !sendingCoder) {
          const it = resp.item;
          const action = String(it.action || "").trim();
          const path = String(it.path || "").trim();
          const result = it.result != null ? JSON.stringify(it.result, null, 2) : "";
          const preview = result && result.length > 1800 ? (result.slice(0, 1800) + "\n...truncated...") : result;
          const msg = [
            "[OBSTRAL] Pending edit approved. Continue without redoing the approved step.",
            `id: ${eid}`,
            action ? `action: ${action}` : "",
            path ? `path: ${path}` : "",
            preview ? ("result:\n" + preview) : "",
          ].filter(Boolean).join("\n");
          // Best-effort: nudge Coder to resume after approval (Lite server pauses tool loops on approvals).
          sendCoder(msg);
        }
      } catch (_) {
      } finally {
        setPendingBusy(false);
        refreshPendingEdits();
      }
    };

    const keyOpenAI =
      status && status.providers && status.providers["openai-compatible"]
        ? !!status.providers["openai-compatible"].api_key_present
        : false;
    const keyMistral = status && status.providers && status.providers.mistral ? !!status.providers.mistral.api_key_present : false;
    const keyCodestral = status && status.providers && status.providers.codestral ? !!status.providers.codestral.api_key_present : false;
    const keyGemini = status && status.providers && status.providers.gemini ? !!status.providers.gemini.api_key_present : false;
    const keyAnthropic =
      status && status.providers && status.providers.anthropic ? !!status.providers.anthropic.api_key_present : false;

    const chatProvider = String(config.provider || "").trim();
    const codeProvider = String(config.codeProvider || "").trim() || chatProvider;
    const observerProvider = String(config.observerProvider || "").trim() || chatProvider;

    const chatKeyPresent = String(chatApiKey || "").trim().length > 0;
    const codeKeyPresent = String(codeApiKey || "").trim().length > 0;
    const observerKeyPresent = String(observerApiKey || "").trim().length > 0;

    const typedKeyFor = (p) => {
      const pp = String(p || "").trim();
      if (!pp) return false;
      if (chatProvider === pp && chatKeyPresent) return true;
      if (codeProvider === pp && codeKeyPresent) return true;
      if (observerProvider === pp && observerKeyPresent) return true;
      if ((chatProvider === pp || codeProvider === pp || observerProvider === pp) && (chatKeyPresent || codeKeyPresent || observerKeyPresent)) return true;
      return false;
    };

    const keyMistralOk = keyMistral || typedKeyFor("mistral");
    const keyCodestralOk = keyCodestral || typedKeyFor("codestral");
    const keyOpenAIOk = keyOpenAI || typedKeyFor("openai-compatible");
    const keyGeminiOk = keyGemini || typedKeyFor("gemini");
    const keyAnthropicOk = keyAnthropic || typedKeyFor("anthropic");

    const KeyDot = ({ ok }) => e("span", { className: "kdot " + (ok ? "ok" : "missing") });

    const serverOutdated =
      status && status.ok && (!status.workspace_root || !status.features);

    const providerLabel = (k) => {
      const p = PROVIDERS.find((x) => x.k === k);
      return (p && p.l && p.l[lang]) || k;
    };

    const modeLabel = (k) => {
      const m = MODES.find((x) => x.k === k);
      return (m && m.l && m.l[lang]) || k;
    };

    const modeUsesCode = (mode) => {
      const m0 = String(mode || "");
      return m0 === "VIBE" || m0.startsWith("diff") || m0 === "ログ解析";
    };

    const coderResolvedProvider = (mode) => (modeUsesCode(mode) ? (String(config.codeProvider || "").trim() || config.provider) : config.provider);
    const coderResolvedModel = (mode) => {
      const useCode = modeUsesCode(mode);
      const m = useCode ? (config.codeModel || config.model) : (config.chatModel || config.model);
      return String(m || "").trim();
    };

    const observerResolvedProvider = () => (String(config.observerProvider || "").trim() || config.provider);
    const observerResolvedModel = () => {
      const m0 = String(config.observerModel || "").trim();
      const m = m0 ? m0 : (config.chatModel || config.model);
      return String(m || "").trim();
    };

    const coderActiveModel = () => coderResolvedModel(config.mode);
    const observerActiveModel = () => observerResolvedModel();

    const shortModel = (m) => {
      const s = String(m || "").trim();
      if (!s) return "";
      return s.length > 28 ? s.slice(0, 28) + "..." : s;
    };

    const scrollBottom = (ref) => {
      const el = ref && ref.current;
      if (!el) return;
      el.scrollTop = el.scrollHeight;
    };

    const applyPreset = (kind) => {
      const p = PRESETS[kind];
      if (!p) return;
      setConfig({ ...DEFAULT_CONFIG, ...p });
    };

    const setProviderSafe = (provider) => {
      const p = String(provider || "").trim();
      let next = { ...config, provider: p };

      if (p === "mistral") {
        next = { ...next, ...PRESETS.vibe };
      } else if (p === "codestral") {
        next = { ...next, ...PRESETS.codestral };
      } else if (p === "mistral-cli") {
        next.provider = "mistral-cli";
        next.baseUrl = "";
        next.model = next.model || "mistral-medium-latest";
        next.chatModel = next.chatModel || next.model;
        next.codeModel = next.codeModel || next.model;
      } else if (p === "openai-compatible") {
        next = { ...next, ...PRESETS.openai };
      } else if (p === "gemini") {
        next = { ...next, ...PRESETS.gemini };
      } else if (p === "anthropic") {
        next = { ...next, ...PRESETS.anthropic };
      } else if (p === "hf") {
        next.provider = "hf";
        next.baseUrl = next.baseUrl || "http://localhost";
        next.model = next.model || "local";
        next.chatModel = next.chatModel || next.model;
        next.codeModel = next.codeModel || next.model;
      }

      setModels([]);
      setModelsErr("");
      setConfig(next);
    };

    const fetchModels = async () => {
      try {
        setModelsLoading(true);
        setModelsErr("");
        const target = String(modelsTarget || "chat");
        const isCode = target === "code";
        const isObs = target === "observer";
        const provider = isCode
          ? (String(config.codeProvider || "").trim() || config.provider)
          : isObs
            ? (String(config.observerProvider || "").trim() || config.provider)
            : config.provider;
        const baseUrl = isCode
          ? (String(config.codeBaseUrl || "").trim() || config.baseUrl)
          : isObs
            ? (String(config.observerBaseUrl || "").trim() || config.baseUrl)
            : config.baseUrl;
        const apiKey = isCode
          ? (String(codeApiKey || "").trim() || String(chatApiKey || "").trim() || String(observerApiKey || "").trim())
          : isObs
            ? (String(observerApiKey || "").trim() || String(chatApiKey || "").trim() || String(codeApiKey || "").trim())
            : (String(chatApiKey || "").trim() || String(codeApiKey || "").trim() || String(observerApiKey || "").trim());
        const j = await postJson("/api/models", {
          provider,
          base_url: strOrUndef(baseUrl),
          api_key: strOrUndef(apiKey),
        });
        const ms = j && Array.isArray(j.models) ? j.models : [];
        setModels(ms);
      } catch (err) {
        setModels([]);
        const m = (err && err.message) ? String(err.message) : String(err || "");
        setModelsErr(m === "Failed to fetch" ? tr(lang, "fetchFailed") : m);
      } finally {
        setModelsLoading(false);
      }
    };

    const createThread = () => {
      setThreadState((s) => {
        const t = makeThread(`${tr(lang, "threads")} #${s.threads.length + 1}`);
        return { threads: [t, ...s.threads], activeId: t.id };
      });
    };

    const renameThread = (id) => {
      const th = threadState.threads.find((t) => t.id === id);
      setEditingTitle(th ? th.title : "");
      setEditingThreadId(id);
    };

    const commitRename = (id) => {
      const title = editingTitle.trim();
      if (title) {
        setThreadState((s) => ({
          ...s,
          threads: s.threads.map((t) => (t.id === id ? { ...t, title, updatedAt: Date.now() } : t)),
        }));
      }
      setEditingThreadId(null);
    };

    const deleteThread = (id) => {
      setConfirmDeleteId(id);
    };

    const confirmDelete = (id) => {
      setConfirmDeleteId(null);
      setThreadState((s) => {
        const remain = s.threads.filter((t) => t.id !== id);
        const nextThreads = remain.length ? remain : [makeThread("Thread 1")];
        const nextActive = nextThreads.some((t) => t.id === s.activeId) ? s.activeId : nextThreads[0].id;
        return { threads: nextThreads, activeId: nextActive };
      });
    };

    const clearActiveThread = () => {
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => (t.id === s.activeId ? { ...t, messages: [], updatedAt: Date.now() } : t)),
      }));
    };

    const exportMd = () => {
      const md = transcriptMd(activeThread, {
        thread: activeThread.title,
        exported_at: new Date().toISOString(),
        provider: config.provider,
        chat_model: config.chatModel || config.model,
        code_model: config.codeModel || config.model,
        base_url: config.baseUrl,
        coder_mode: config.mode,
        coder_persona: config.persona,
        coder_model_selected: coderActiveModel(),
        observer_mode: config.observerMode,
        observer_persona: config.observerPersona,
        observer_intensity: config.observerIntensity,
        observer_model_selected: observerActiveModel(),
      });
      const safe = String(activeThread.title || "thread").replace(/[\\/:*?\"<>|]/g, "_");
      downloadText(`obstral-${safe}.md`, md);
    };

    const copyText = async (text, id) => {
      try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          await navigator.clipboard.writeText(String(text || ""));
          if (id) { setCopiedId(id); setTimeout(() => setCopiedId(null), 1400); }
          return;
        }
      } catch (_) {}
      window.prompt(tr(lang, "copy"), String(text || ""));
    };

    const runCmd = async (msgId, blockIdx, langHint, codeText) => {
      setExecResults(prev => ({
        ...prev,
        [msgId]: { ...(prev[msgId] || {}), [blockIdx]: { stdout: "", stderr: "", exit_code: 0, running: true } },
      }));
      try {
        const cwd = resolvedCwd(config.toolRoot, activeThread && activeThread.id, activeThread && activeThread.workdir);
        const command = normalizeExecScript(langHint, codeText);
        const danger = dangerousCommandReason(command);
        if (danger && !confirmDangerous(lang, danger)) {
          setExecResults(prev => ({
            ...prev,
            [msgId]: { ...(prev[msgId] || {}), [blockIdx]: { stdout: "", stderr: "cancelled", exit_code: -2, running: false } },
          }));
          return;
        }
        const result = await postJson("/api/exec", { command, cwd });
        setExecResults(prev => ({
          ...prev,
          [msgId]: { ...(prev[msgId] || {}), [blockIdx]: {
            stdout: String(result.stdout || ""),
            stderr: String(result.stderr || ""),
            exit_code: typeof result.exit_code === "number" ? result.exit_code : -1,
            running: false,
          }},
        }));
      } catch (err) {
        setExecResults(prev => ({
          ...prev,
          [msgId]: { ...(prev[msgId] || {}), [blockIdx]: {
            stdout: "", stderr: String(err.message || "error"), exit_code: -1, running: false,
          }},
        }));
      }
    };

    const openFile = async (path) => {
      try {
        const cwd = resolvedCwd(config.toolRoot, activeThread && activeThread.id, activeThread && activeThread.workdir);
        await postJson("/api/open", { path, cwd });
      } catch (_) {}
    };

    const renderMessage = (m) => {
      const canExec = !!(status && status.features && status.features.exec);
      const canOpen = !!(status && status.features && status.features.open_file);
      const raw = String(m && m.content ? m.content : "");
      const pane = (m && (m.pane === "observer" || m.pane === "chat" || m.pane === "coder")) ? m.pane : "coder";
      const s =
        (!m.streaming && m.role === "assistant" && pane === "observer")
          ? stripObserverMeta(raw)
          : raw;
      const isLong = !m.streaming && (s.length > 2600 || (s.match(/\n/g) || []).length > 40);
      const isExpanded = expandedMsgs.has(m.id);
      const isCollapsed = isLong && !isExpanded;
      const choices = (!m.streaming && m.role === "assistant" && m.pane !== "observer") ? extractChoices(s) : [];
      const streamingNode = e("span", null,
        s || e("span", { className: "thinking" }, tr(lang, "streaming")),
        e("span", { className: "cursor-blink" }, "▊")
      );
      // File chips: shown below completed Coder assistant messages when open_file is supported.
      const isCoderAsst = !m.streaming && m.role === "assistant" && m.pane !== "observer" && m.pane !== "chat";
      const fileChips = (canOpen && isCoderAsst) ? extractPathHints(s, 10) : [];
      return e(
        "div",
        { key: m.id, className: "msg msg-pane-" + pane + (m.role === "user" ? " msg-user" : " msg-assistant") + (m.pane === "chat" ? " chat-msg" : "") },
        e("div", { className: "avatar" }, avatarLabel(m)),
        e(
          "div",
          { className: "bubble " + (m.role === "user" ? "user" : "assistant") + " pane-" + pane },
          e(
            "div",
            { className: "msg-meta" },
            e("div", { className: "who" }, whoLabel(m, lang)),
            m.ts ? e("span", { className: "msg-ts" }, relativeTime(m.ts, lang)) : null,
            e("div", { className: "mini" },
              isLong && e("button", {
                onClick: () => setExpandedMsgs((prev) => {
                  const next = new Set(prev);
                  if (next.has(m.id)) next.delete(m.id); else next.add(m.id);
                  return next;
                }),
              }, isExpanded ? tr(lang, "less") : tr(lang, "more")),
              (!m.streaming && m.role === "assistant" && (m.pane === "observer" || isLong)) ? e("button", {
                onClick: () => setReaderModal({
                  id: m.id,
                  title: whoLabel(m, lang) + (activeThread && activeThread.title ? (" · " + activeThread.title) : ""),
                  ts: m.ts,
                  content: String(m.content || ""),
                }),
              }, tr(lang, "reader")) : null,
              e("button", {
                className: copiedId === m.id ? "copied" : "",
                onClick: () => copyText(m.content || "", m.id),
              }, copiedId === m.id ? "✓" : tr(lang, "copy"))
            )
          ),
          e(
            "div",
            { className: "content" + (isCollapsed ? " content-collapsed" : "") },
            m.streaming ? streamingNode : renderWithThink(
              s,
              execResults[m.id] || {},
              canExec ? ((blockIdx, langHint, codeText) => runCmd(m.id, blockIdx, langHint, codeText)) : null,
              canOpen ? openFile : null
            ),
            choices && choices.length
              ? e(
                  "div",
                  { className: "choice-row" },
                  choices.map((c) =>
                    e(
                      "button",
                      {
                        key: "ch" + String(c.n),
                        className: "choice-btn",
                        type: "button",
                        onClick: () => sendCoder(choiceReply(lang, c.n, c.text)),
                      },
                      String(c.n) + ") " + String(c.text).slice(0, 42)
                    )
                  )
                )
              : null
          ),
          fileChips.length ? e(
            "div",
            { className: "file-chips" },
            fileChips.map((p) =>
              e("button", {
                key: p,
                className: "file-chip",
                title: p,
                onClick: () => openFile(p),
              }, "📂 " + p)
            )
          ) : null
        )
      );
    };

    const appendDelta = (threadId, msgId, delta, bodyRef) => {
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => {
          if (t.id !== threadId) return t;
          return {
            ...t,
            updatedAt: Date.now(),
            messages: (t.messages || []).map((m) =>
              m.id === msgId ? { ...m, content: String(m.content || "") + String(delta || "") } : m
            ),
          };
        }),
      }));
      requestAnimationFrame(() => scrollBottom(bodyRef));
    };

    const setMsg = (threadId, msgId, content, bodyRef) => {
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => {
          if (t.id !== threadId) return t;
          return {
            ...t,
            updatedAt: Date.now(),
            messages: (t.messages || []).map((m) => (m.id === msgId ? { ...m, content, streaming: false } : m)),
          };
        }),
      }));
      requestAnimationFrame(() => scrollBottom(bodyRef));
    };

    const ensureTitle = (threadId, userText) => {
      const autoPrefix = tr(lang, "threads") + " #";
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => {
          if (t.id !== threadId) return t;
          const title = String(t.title || "");
          const isAuto = title === "Untitled" || /^Thread\\s+\\d+/.test(title) || title.indexOf(autoPrefix) === 0;
          if (!isAuto) return t;
          return { ...t, title: titleFrom(userText), updatedAt: Date.now() };
        }),
      }));
    };

    const stopCoder = () => {
      if (abortCoderRef.current) abortCoderRef.current.abort();
      abortCoderRef.current = null;
      setSendingCoder(false);
    };

    const stopObserver = () => {
      if (abortObserverRef.current) abortObserverRef.current.abort();
      abortObserverRef.current = null;
      setSendingObserver(false);
    };

    const prettyErr = (err) => {
      const m = (err && err.message) ? String(err.message) : String(err || "");
      return m === "Failed to fetch" ? tr(lang, "fetchFailed") : (m || tr(lang, "error"));
    };

    const paneMessages = (pane) => {
      const all = (activeThread && activeThread.messages) ? activeThread.messages : [];
      if (pane === "observer") return all.filter((m) => m.pane === "observer");
      if (pane === "chat") return all.filter((m) => m.pane === "chat");
      return all.filter((m) => m.pane !== "observer" && m.pane !== "chat");
    };

    const finishStreaming = (threadId, msgId) => {
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => {
          if (t.id !== threadId) return t;
          return {
            ...t,
            updatedAt: Date.now(),
            messages: (t.messages || []).map((m) => (m.id === msgId ? { ...m, streaming: false } : m)),
          };
        }),
      }));
    };

    const extractCodeBlocks = (text, maxBlocks, maxCharsEach) => {
      const s = String(text || "");
      const out = [];
      const re = /```[^\n]*\n([\s\S]*?)```/g;
      let m;
      while ((m = re.exec(s)) !== null) {
        if (out.length >= maxBlocks) break;
        const body = String(m[1] || "").trim();
        if (!body) continue;
        out.push(body.length > maxCharsEach ? body.slice(0, maxCharsEach) + "..." : body);
      }
      return out;
    };

    const extractPathHints = (text, maxItems) => {
      const s = String(text || "");
      const re = /(?:[A-Za-z]:\\\\[^\s"'`]+|(?:\.\.?\/)?[A-Za-z0-9._-]+(?:\/[A-Za-z0-9._-]+)+\.[A-Za-z0-9._-]+)/g;
      const uniq = {};
      const out = [];
      let m;
      while ((m = re.exec(s)) !== null) {
        const p = String(m[0] || "").trim();
        if (!p || uniq[p]) continue;
        uniq[p] = true;
        out.push(p);
        if (out.length >= maxItems) break;
      }
      return out;
    };

    const buildCliTemplate = () => {
      return [
        "[CLI coding task template]",
        "You are a CLI coding agent.",
        "Work autonomously in long-run mode: decompose into modules, edit files, and verify.",
        "Prioritize concrete file changes over abstract discussion.",
        "Always operate under tool_root (create a new project dir; do NOT touch OBSTRAL's repo).",
        "Before each major action, write a 3-line scratchpad: goal / risk / next (keep it short).",
        "If an action requires approval, STOP and tell the user which pending edit id(s) to approve, then continue after approval.",
        "Never run destructive cleanup commands (git reset --hard / git clean -fd / git rm ... .) unless the user explicitly asks.",
        "At the end, report: changed files / why / remaining risks.",
      ].join("\n");
    };

    const insertCliTemplate = () => {
      const t = buildCliTemplate();
      setCoderInput((prev) => {
        const p = String(prev || "").trim();
        return p ? (p + "\n\n" + t) : t;
      });
    };

    const coderContextPacket = () => {
      const msgs = paneMessages("coder");
      let lastUser = null;
      let lastAsst = null;
      const recentAsst = [];
      const recentUser = [];
      for (let i = msgs.length - 1; i >= 0; i--) {
        const m = msgs[i];
        if (m.role === "assistant") {
          if (!lastAsst) lastAsst = m;
          if (recentAsst.length < 4) recentAsst.push(m);
        } else if (m.role === "user") {
          if (!lastUser) lastUser = m;
          if (recentUser.length < 3) recentUser.push(m);
        }
      }

      const cut = (t, n) => {
        const s = String(t || "").trim();
        return s.length > n ? s.slice(0, n) + "..." : s;
      };

      const codeBlocks = [];
      recentAsst.forEach((m) => {
        extractCodeBlocks(m.content, 2, 900).forEach((b) => {
          if (codeBlocks.length < 6) codeBlocks.push(b);
        });
      });

      const pathHints = [];
      [...recentAsst, ...recentUser].forEach((m) => {
        extractPathHints(m.content, 12).forEach((p) => {
          if (pathHints.length < 12 && pathHints.indexOf(p) === -1) pathHints.push(p);
        });
      });

      const parts = [];
      parts.push("--- coder_context ---");
      parts.push("observer_view: You can inspect coder outputs and code snippets below.");
      parts.push(`coder_mode: ${String(config.mode || "")}`);
      parts.push(`coder_model: ${coderActiveModel()}`);
      const cwdNow = resolvedCwd(config.toolRoot, activeThread && activeThread.id, activeThread && activeThread.workdir);
      parts.push(`tool_root: ${cwdNow ? String(cwdNow) : ""}`);
      if (lastUser) parts.push("last_user:\n" + cut(lastUser.content, 1200));
      if (lastAsst) parts.push("last_assistant:\n" + cut(lastAsst.content, 2600));
      if (pathHints.length) parts.push("file_hints:\n- " + pathHints.join("\n- "));
      if (codeBlocks.length) {
        parts.push(
          "recent_code_blocks:\n" +
          codeBlocks.map((b, i) => `#${i + 1}\n\`\`\`\n${b}\n\`\`\``).join("\n\n")
        );
      }
      if (diff && String(diff).trim()) parts.push("diff: (present)");
      return parts.join("\n");
    };

    const runCoderAgentic = async (text, threadId, asstMsgId, reqCfg, resolvedKey, history, ac, threadWorkdir) => {
      const autonomy = String((reqCfg && reqCfg.autonomy) || "longrun").trim().toLowerCase();
      const longrun = autonomy !== "off";
      const MAX_ITERS = longrun ? 14 : 8;
      const TRUNC_STDOUT = 2000;
      const TRUNC_STDERR = 800;
      const KEEP_TOOL_TURNS = longrun ? 6 : 3;
      const WANTS_REPO_GOAL = /(?:\brepo\b|\brepository\b|\bgit\b|scaffold|bootstrap|init|setup|create\s+(?:a\s+)?repo|create\s+(?:a\s+)?repository|リポ|リポジトリ|雛形|ひな形|プロジェクト|git\s+init)/i.test(String(text || ""));
      let goalChecks = 0;

      const truncTool = (s, max) => {
        const t = String(s || "").trim();
        if (t.length <= max) return t;
        const lines = t.split("\n").length;
        return t.slice(0, max) + `\n[…truncated — ${lines} lines total, first ${max} chars shown]`;
      };

      const pruneToolMessages = (msgs) => {
        const toolIdxs = msgs.reduce((acc, m, i) => m.role === "tool" ? [...acc, i] : acc, []);
        if (toolIdxs.length <= KEEP_TOOL_TURNS) return;
        const toPrune = toolIdxs.slice(0, toolIdxs.length - KEEP_TOOL_TURNS);
        for (const idx of toPrune) {
          const content = String(msgs[idx].content || "");
          // Never prune failures: they are the most important recovery context.
          if (!content.trimStart().startsWith("OK (exit_code: 0)")) continue;
          const lines = content.split("\n");
          if (lines.length > 2) {
            msgs[idx] = { ...msgs[idx], content: lines[0] + ` [pruned ${lines.length}L]` };
          }
        }
      };

      // IMPORTANT: `exec` runs in a fresh process each time — `cd` does NOT persist across tool calls.
      // Track the agent's current working directory and pass it as `cwd` on every exec/write_file.
      // This prevents nested-git disasters and "why did it run in the repo root?" failures.
      const threadRoot = resolvedThreadRoot(config.toolRoot, threadId);
      const threadRootNorm = threadRoot ? normalizePathSep(String(threadRoot)).replace(/\/+$/g, "") : "";
      let curWorkdir = safeWorkdir(threadWorkdir);

      const cwdNow = () => resolvedCwd(config.toolRoot, threadId, curWorkdir);
      const cwdLabelNow = () => {
        const c = cwdNow();
        return c ? String(c) : "(workspace root)";
      };

      const setWorkdir = (nextWorkdir) => {
        const safe = safeWorkdir(nextWorkdir);
        if (safe === curWorkdir) return;
        curWorkdir = safe;
        // Persist to thread state so subsequent manual runs start in the right directory.
        setThreadState((s) => ({
          ...s,
          threads: s.threads.map((t) => (
            t.id === threadId ? { ...t, updatedAt: Date.now(), workdir: safe } : t
          )),
        }));
      };

      const PWD_MARKER = "__OBSTRAL_PWD__=";
      const wrapExecWithPwd = (cmd) => {
        const raw = String(cmd || "").trim();
        if (!raw) return raw;
        if (isWindowsHost()) {
          // Always emit a trailing marker so the UI can update curWorkdir even when `cd` is used.
          // Use try/finally so we still emit on most failures (helps stuck recovery).
          return [
            "$ErrorActionPreference = 'Stop'",
            "try {",
            raw,
            "} finally {",
            `Write-Output ("${PWD_MARKER}" + (Get-Location).Path)`,
            "}",
          ].join("\n");
        }
        // POSIX: keep behavior simple (do not `set -e`).
        return raw + `\necho "${PWD_MARKER}$(pwd)"`;
      };

      const stripPwdMarker = (stdoutRaw) => {
        const raw = String(stdoutRaw || "");
        if (!raw) return { stdout: "", pwd: "" };
        const lines = raw.replace(/\r\n/g, "\n").split("\n");
        let pwd = "";
        const kept = [];
        for (const l of lines) {
          if (l.startsWith(PWD_MARKER)) {
            pwd = l.slice(PWD_MARKER.length).trim();
            continue;
          }
          kept.push(l);
        }
        return { stdout: kept.join("\n").trimEnd(), pwd };
      };

      const maybeUpdateWorkdirFromPwd = (pwdAfter) => {
        const p0 = String(pwdAfter || "").trim();
        if (!p0 || !threadRootNorm) return;
        const p = normalizePathSep(p0).replace(/\/+$/g, "");
        const idx = p.toLowerCase().lastIndexOf(threadRootNorm.toLowerCase());
        if (idx === -1) return;
        const rel0 = p.slice(idx + threadRootNorm.length).replace(/^\/+/g, "");
        const safeRel = safeWorkdir(rel0);
        // If rel0 is non-empty but unsafe, ignore (avoid escaping tool_root).
        if (rel0 && !safeRel) return;
        setWorkdir(safeRel);
      };

      const sandboxBreachReason = (pwdAfter) => {
        // Detect if a command ended outside tool_root. This is a common cause of nested-git disasters.
        const p0 = String(pwdAfter || "").trim();
        if (!p0 || !threadRootNorm) return "";
        const p = normalizePathSep(p0).replace(/\/+$/g, "");
        const idx = p.toLowerCase().lastIndexOf(threadRootNorm.toLowerCase());
        if (idx === -1) return `cwd_after escaped tool_root: ${p0}`;
        return "";
      };

      // UI-side loop breaker: if the model repeats the exact same failing command,
      // block it and force a different strategy.
      const cmdStats = new Map(); // key -> { attempts, fails, lastErr }
      const cmdKey = (cmd) => String(cmd || "").toLowerCase().replace(/\s+/g, " ").trim();
      const cmdSig = (stderr, stdout) => {
        const s = String(stderr || "") || String(stdout || "");
        const first = (s.split("\n")[0] || "").trim();
        return first.slice(0, 180);
      };
      const blockedByRepeatFailure = (k) => {
        const st = cmdStats.get(k);
        if (!st) return "";
        if ((st.fails || 0) >= 2) {
          const why = st.lastErr ? ` last_error: ${st.lastErr}` : "";
          return `repeated failure (${st.fails}x).${why}`;
        }
        return "";
      };
      const noteCmd = (k, failed, sig) => {
        const st = cmdStats.get(k) || { attempts: 0, fails: 0, lastErr: "" };
        st.attempts++;
        if (failed) {
          st.fails++;
          if (sig) st.lastErr = sig;
        } else {
          st.fails = 0;
          st.lastErr = "";
        }
        cmdStats.set(k, st);
        return st;
      };

      const fnv1a64 = (s) => {
        // FNV-1a 64-bit (BigInt) hash. Cheap and stable for stuck detection.
        let h = 0xcbf29ce484222325n;
        const prime = 0x100000001b3n;
        const str = String(s || "");
        for (let i = 0; i < str.length; i++) {
          h ^= BigInt(str.charCodeAt(i) & 0xff);
          h = (h * prime) & 0xffffffffffffffffn;
        }
        return h;
      };

      const governor = {
        consecutiveFailures: 0,
        lastErrSig: "",
        sameErrRepeats: 0,
        lastOutHash: 0n,
        sameOutRepeats: 0,
        pendingHint: "",
      };

      const deriveGovernorHint = (stderr, stdout) => {
        const sErr = String(stderr || "");
        const sOut = String(stdout || "");
        const s = (sErr || sOut || "").toLowerCase();
        if (!s) return "";

        // Poison proxy: github push/connect fails via 127.0.0.1:9
        if (s.includes("port 443 via 127.0.0.1") || s.includes("127.0.0.1:9")) {
          return [
            "Detected a poisoned proxy env (127.0.0.1:9).",
            "Fix (PowerShell):",
            "$env:HTTP_PROXY=''; $env:HTTPS_PROXY=''; $env:ALL_PROXY=''; $env:GIT_HTTP_PROXY=''; $env:GIT_HTTPS_PROXY=''",
            "Then retry. For GitHub push, prefer: .\\scripts\\push_ssh.ps1 (SSH over 443).",
            "Alternative: .\\scripts\\push.ps1 (with $env:GITHUB_TOKEN).",
          ].join("\n");
        }

        // WDAC-ish: msys tools like head.exe fail with Win32 error 5.
        if (s.includes("win32 error 5") && s.includes("head.exe")) {
          return [
            "This environment blocks some MSYS/Unix tools (Win32 error 5).",
            "Avoid `head`, `sed`, `nl`, pipes into MSYS tools. Use PowerShell equivalents:",
            "Get-Content file | Select-Object -First 40",
          ].join("\n");
        }

        // Cargo exe lock (binary is running).
        if (s.includes("failed to remove file") && s.includes("obstral.exe") && (s.includes("access is denied") || s.includes("アクセスが拒否"))) {
          return [
            "obstral.exe is locked (running). Stop the process before rebuilding.",
            "Fix: .\\scripts\\kill-obstral.ps1 ; then re-run build (or use .\\scripts\\run-tui.ps1 / run-ui.ps1).",
          ].join("\n");
        }

        // Embedded repo hint (also covered elsewhere).
        if (s.includes("embedded git repository") || s.includes("does not have a commit checked out")) {
          return [
            "You are mixing repos (nested git repo).",
            "Fix: operate inside the project directory only, or move it under tool_root (.tmp/<threadId>).",
            "Do NOT run `git add .` from the OBSTRAL repo root.",
          ].join("\n");
        }

        return "";
      };

      const isWindows = isWindowsHost();
      const SYSTEM_BASE = isWindows ? [
        "You are an autonomous coding agent with DIRECT access to the user's Windows machine.",
        `Working directory (tool_root): ${cwdLabelNow()}. Always create new projects under this directory. Do NOT cd to parent directories.`,
        "CRITICAL RULES — follow these without exception:",
        "0. NEVER create a git repo inside another git repo. If you see 'embedded git repository' warnings, STOP and relocate to a clean directory under tool_root.",
        "1. ALWAYS use tools to act: use `write_file` to create/edit files, and `exec` to run commands. NEVER just show code.",
        "   Fallback (if tool calls are not supported): output ONE ```powershell``` code block containing ONLY commands (no `$ ` or `PS>` prompts).",
        "2. Use PowerShell syntax ONLY (cmd.exe is NOT used):",
        "   - Create directory tree: New-Item -ItemType Directory -Force -Path 'a/b/c'",
        "   - Create file with content: Set-Content -Path 'file.txt' -Value 'line1`nline2' -Encoding UTF8",
        "   - Multi-line file: @('line1','line2') | Set-Content -Path 'file.txt' -Encoding UTF8",
        "   - Append to file: Add-Content -Path 'file.txt' -Value 'more' -Encoding UTF8",
        "   - Git (new repo): New-Item -ItemType Directory -Force -Path 'MyRepo'; cd 'MyRepo'; git init; git add .; git commit -m 'init'",
        "   - NEVER use mkdir -p, touch, cat >, or any Unix syntax.",
        "3. Execute ALL steps immediately via exec. Do NOT ask for permission or confirmation.",
        "4. After each exec call, read the output and continue until the task is 100% complete.",
        "5. End with a brief summary listing every file created/modified and any remaining steps.",
      ].join("\n") : [
        "You are an autonomous coding agent with DIRECT access to the user's local machine.",
        `Working directory (tool_root): ${cwdLabelNow()}. Always create new projects under this directory. Do NOT cd to parent directories.`,
        "CRITICAL RULES — follow these without exception:",
        "0. NEVER create a git repo inside another git repo. If you see 'embedded git repository' warnings, STOP and relocate to a clean directory under tool_root.",
        "1. ALWAYS use tools to act: use `write_file` to create/edit files, and `exec` to run commands. NEVER just show code.",
        "   Fallback (if tool calls are not supported): output ONE ```bash``` code block containing ONLY commands (no `$ ` prompts).",
        "2. Use Unix shell commands:",
        "   - Create directory: mkdir -p path/to/dir",
        "   - Write file: printf '%s' 'content' > file.txt   OR   python3 -c \"open('f','w').write('...')\"",
        "   - Multi-line file: use a heredoc via python3 or printf with \\n",
        "   - Git: git init, git add ., git commit -m 'init'",
        "3. Execute ALL steps immediately via exec. Do NOT ask for permission or confirmation.",
        "4. After each exec call, read the output and continue until the task is 100% complete.",
        "5. End with a brief summary listing every file created/modified and any remaining steps.",
      ].join("\n");

      const SYSTEM_REASONING = [
        "",
        "[Planning Protocol — emit ONCE before your very first exec call]",
        "<plan>",
        "goal: <one sentence: what the finished task looks like when done>",
        "steps: 1) ... 2) ... 3) ... (3-7 concrete, ordered steps)",
        "risks: <the 2 most likely failure modes for this specific task>",
        "assumptions: <what you are taking as given>",
        "</plan>",
        "",
        "[Reasoning Protocol — emit before EVERY exec call]",
        "<think>",
        "goal: <≤12 words: what must succeed right now>",
        "risk: <≤12 words: most likely failure mode>",
        "doubt: <≤12 words: one reason this could be wrong>",
        "next: <≤12 words: exact command or step>",
        "verify: <≤12 words: how to confirm this step succeeded>",
        "</think>",
        "This 5-line check (~50 tokens) prevents wrong-direction errors that cost 300+ tokens to recover.",
        "",
        "[Error Protocol]",
        "If exit_code ≠ 0: STOP immediately.",
        "  1. Quote the exact error line.",
        "  2. State root cause in one sentence.",
        "  3. Fix with one corrected command.",
        "If the SAME approach fails 3 consecutive times: STOP, explain why,",
        "  and propose a completely different strategy. Never repeat a failing command.",
      ].join("\n");

      const SYSTEM_BASE_TEXT = SYSTEM_BASE + SYSTEM_REASONING;

      const execTool = {
        type: "function",
        function: {
          name: "exec",
          description: isWindows
            ? "Execute a PowerShell command on the user's Windows machine. Use for ALL local operations: create directories (New-Item -ItemType Directory -Force), write files (Set-Content -Encoding UTF8), run programs, git commands, etc. Check exit_code in the result — non-zero means failure, diagnose and retry."
            : "Execute a shell command on the user's local machine. Use for ALL local operations: create directories (mkdir -p), write files (tee/printf), run programs, git commands, etc. Check exit_code in the result — non-zero means failure, diagnose and retry.",
          parameters: {
            type: "object",
            properties: { command: { type: "string", description: "The exact command to run" } },
            required: ["command"],
          },
        },
      };

      const writeFileTool = {
        type: "function",
        function: {
          name: "write_file",
          description: isWindows
            ? "Write a UTF-8 text file under tool_root. Use this tool for ALL file creation/edits. Provide a relative path (no drive letters, no '..')."
            : "Write a text file under tool_root. Use this tool for ALL file creation/edits. Provide a relative path (no '..').",
          parameters: {
            type: "object",
            properties: {
              path: { type: "string", description: "Relative file path under tool_root (e.g. src/main.py)" },
              content: { type: "string", description: "Full file content (UTF-8 text)" },
            },
            required: ["path", "content"],
          },
        },
      };

      const messages = [
        { role: "system", content: SYSTEM_BASE_TEXT },
        ...history,
        { role: "user", content: text },
      ];

      let display = "";
      let awaitingApproval = false;
      const flush = (extra) => {
        setMsg(threadId, asstMsgId, (display + (extra || "")).trim() || "…", coderBodyRef);
      };

      // SSE streaming helper — streams token deltas in real-time, returns finish event.
      const streamChatTools = async (payload, signal, onDelta) => {
        const resp = await fetch("/api/chat_tools_stream", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
          signal,
        });
        if (!resp.ok) {
          const errBody = await resp.json().catch(() => ({ error: `HTTP ${resp.status}` }));
          throw new Error(errBody.error || `HTTP ${resp.status}`);
        }
        const reader = resp.body.getReader();
        const decoder = new TextDecoder();
        let sseBuffer = "";
        let fullText = "";
        let finishReason = "stop";
        let toolCalls = [];
        try {
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            sseBuffer += decoder.decode(value, { stream: true });
            let sep;
            while ((sep = sseBuffer.indexOf("\n\n")) !== -1) {
              const block = sseBuffer.slice(0, sep);
              sseBuffer = sseBuffer.slice(sep + 2);
              let evtType = "message", evtData = "";
              for (const line of block.split("\n")) {
                if (line.startsWith("event: ")) evtType = line.slice(7).trim();
                else if (line.startsWith("data: ")) evtData = line.slice(6);
              }
              if (evtType === "delta") {
                try { const { delta } = JSON.parse(evtData); if (delta) { fullText += delta; onDelta(delta); } } catch (_) {}
              } else if (evtType === "finish") {
                try { const f = JSON.parse(evtData); finishReason = f.finish_reason || "stop"; toolCalls = f.tool_calls || []; } catch (_) {}
              } else if (evtType === "error") {
                let msg = "stream error";
                try { msg = JSON.parse(evtData).error || msg; } catch (_) {}
                throw new Error(msg);
              }
            }
          }
        } finally {
          reader.releaseLock();
        }
        return { text: fullText, finishReason, toolCalls };
      };

      const extractImpliedExecScripts = (txt) => {
        const src = String(txt || "");
        const out = [];
        const fenceRe = /```([a-zA-Z0-9_-]+)?\n([\s\S]*?)```/g;
        let m;
        while ((m = fenceRe.exec(src)) !== null) {
          const langHint = String(m[1] || "").trim().toLowerCase();
          const body = String(m[2] || "").trim();
          if (!body) continue;
          const isShellLang = /^(bash|sh|shell|zsh|powershell|pwsh|ps1|ps|console)$/.test(langHint);
          const looksLikeCmd = /(New-Item\b|Set-Content\b|Add-Content\b|Remove-Item\b|Copy-Item\b|Move-Item\b|\bmkdir\b|\bcd\b|\bgit\b|\bcargo\b|\bpython\b|\bnode\b|\bnpm\b|\bpnpm\b|\byarn\b)/i.test(body);
          if (isShellLang || looksLikeCmd) {
            out.push({ langHint, script: body });
            if (out.length >= 3) break;
          }
        }
        return out;
      };

      for (let iter = 0; iter < MAX_ITERS; iter++) {
        if (ac.signal.aborted) break;
        pruneToolMessages(messages);
        // One-shot governor hint injection (outer-loop behavioral control).
        // Also inject periodic progress checkpoints in longrun mode so the agent doesn't drift.
        let govHint = governor.pendingHint ? String(governor.pendingHint || "").trim() : "";
        if (longrun && (iter === 3 || iter === 6 || iter === 9 || iter === 12)) {
          const cp = [
            "Progress checkpoint:",
            "- State: DONE / REMAINING / ON_TRACK.",
            "- If REMAINING: list the next 1-3 concrete commands.",
            "- If stuck: run diagnostics (pwd/ls/git status) before changing strategy.",
          ].join("\n");
          govHint = govHint ? (govHint + "\n\n" + cp) : cp;
        }
        messages[0] = {
          role: "system",
          content: govHint
            ? (SYSTEM_BASE_TEXT + "\n\n[Governor]\n" + govHint)
            : SYSTEM_BASE_TEXT,
        };
        governor.pendingHint = "";

        let streamResult;
        try {
          // Separate text tokens from previous turn with a blank line.
          if (display) display += "\n\n";
           streamResult = await streamChatTools({
             messages,
            tools: [execTool, writeFileTool],
             model: String(reqCfg.codeModel || reqCfg.model || ""),
             base_url: String(reqCfg.baseUrl || ""),
             api_key: resolvedKey || undefined,
             temperature: numOrUndef(reqCfg.temperature),
            max_tokens: numOrUndef(reqCfg.maxTokens) || 4096,
            timeout_seconds: numOrUndef(reqCfg.timeoutSeconds),
          }, ac.signal, (delta) => {
            display += delta;
            flush();
          });
        } catch (err) {
          if (ac.signal.aborted) break;
          display += `[error] ${err.message}`;
          flush();
          break;
        }

        const { text: asstText, finishReason, toolCalls: asstToolCalls } = streamResult;

        // Append assistant turn to conversation history (OpenAI format).
        const asstMsg = { role: "assistant", content: asstText || null };
        if (asstToolCalls.length > 0) asstMsg.tool_calls = asstToolCalls;
        messages.push(asstMsg);

        if (finishReason === "tool_calls" && asstToolCalls.length > 0) {
          for (const tc of asstToolCalls) {
            if (ac.signal.aborted) break;
            if (tc.type !== "function" || !tc.function || !tc.function.name) continue;

            const toolName = String(tc.function.name || "").trim();

            if (toolName === "exec") {
              let args;
              try { args = JSON.parse(tc.function.arguments || "{}"); } catch (_) { args = {}; }
              const command = stripShellTranscript(String(args.command || ""));
              const commandToRun = normalizeExecScript("", command);
              const k = cmdKey(commandToRun);
              const repeatBlock = blockedByRepeatFailure(k);
              const danger = dangerousCommandReason(commandToRun);

              const win = isWindowsHost();
              const fenceLang = win ? "powershell" : "bash";
              const prompt = win ? "PS> " : "$ ";
              display += (display ? "\n\n" : "") + "```" + fenceLang + "\n" + prompt + command;
              flush("\n```");

              let toolResult;
              try {
                if (repeatBlock) {
                  toolResult = `error: blocked repeated failing command (${repeatBlock}). You MUST choose a different command/strategy.`;
                  display += `\n(blocked: ${repeatBlock})\n\`\`\`\nexit: -1`;
                  flush();
                  messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
                  continue;
                }
                if (danger) {
                  toolResult = `error: blocked dangerous command (${danger}). Ask the user to run it manually if truly intended.`;
                  display += `\n(blocked: ${danger})\n\`\`\`\nexit: -1`;
                  flush();
                  messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
                  continue;
                }
                if (config.requireCommandApproval && !window.confirm("Run command?\n\n" + commandToRun)) {
                  toolResult = "error: command rejected by user";
                  display += "\n(rejected)\n```\nexit: -1";
                  flush();
                  messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
                  continue;
                }

                const cwdUsed = cwdNow();
                const execCmd = wrapExecWithPwd(commandToRun);
                const execRes = await postJson("/api/exec", { command: execCmd, cwd: cwdUsed }, ac.signal);
                const parsed = stripPwdMarker(execRes.stdout);
                const breach = sandboxBreachReason(parsed.pwd);
                maybeUpdateWorkdirFromPwd(parsed.pwd);
                const stdout = truncTool(parsed.stdout, TRUNC_STDOUT);
                const stderr = truncTool(execRes.stderr, TRUNC_STDERR);
                const exitCode = execRes.exit_code;
                const looksHardError = (t) => /(^|\n)\s*(fatal:|error:|exception|traceback)\b/i.test(String(t || ""));
                const failed = exitCode !== 0 || looksHardError(stderr) || !!breach;
                noteCmd(k, failed, cmdSig(stderr, stdout));
                const hintGit = failed ? gitRepoHint(stderr) : "";
                const hintGov = failed ? deriveGovernorHint(stderr, stdout) : "";
                const hintSandbox = breach ? [
                  "SANDBOX BREACH: command ended outside tool_root.",
                  breach,
                  "Fix: re-run under tool_root; avoid `cd ..` / absolute paths. Verify `pwd` stays under tool_root.",
                ].join("\n") : "";
                const hint = [hintGit, hintGov, hintSandbox].filter(Boolean).join("\n\n");

                const cwdUsedLabel = cwdUsed ? String(cwdUsed) : "(workspace root)";
                const cwdAfter = cwdNow();
                const cwdAfterLabel = cwdAfter ? String(cwdAfter) : "(workspace root)";
                const cwdLine = cwdUsedLabel === cwdAfterLabel
                  ? `cwd: ${cwdUsedLabel}`
                  : `cwd: ${cwdUsedLabel}\ncwd_after: ${cwdAfterLabel}`;

                if (failed) {
                  governor.consecutiveFailures++;
                  const sig0 = cmdSig(stderr, stdout);
                  const sig = sig0 || (breach ? String(breach).slice(0, 180) : "");
                  if (sig && sig === governor.lastErrSig) governor.sameErrRepeats++;
                  else { governor.lastErrSig = sig; governor.sameErrRepeats = 1; }
                  const outHash = fnv1a64(String(stderr || "") + "\n" + String(stdout || ""));
                  if (outHash === governor.lastOutHash) governor.sameOutRepeats++;
                  else { governor.lastOutHash = outHash; governor.sameOutRepeats = 1; }

                  // Escalate: if we're stuck, inject a hint to force a strategy change.
                  if (governor.sameErrRepeats >= 2 || governor.sameOutRepeats >= 2 || governor.consecutiveFailures >= 3) {
                    const stuck = [
                      "You are stuck in a failure loop.",
                      governor.lastErrSig ? ("last_error_signature: " + governor.lastErrSig) : "",
                      "STOP repeating the same approach. Change strategy.",
                      hintGov || hintGit || "",
                      "First verify cwd/tool_root, then pick a different command.",
                    ].filter(Boolean).join("\n");
                    governor.pendingHint = breach ? (hintSandbox + "\n\n" + stuck) : stuck;
                  } else if (breach) {
                    // Sandbox breaches are always critical: force a correction immediately.
                    governor.pendingHint = hintSandbox;
                  }
                } else {
                  governor.consecutiveFailures = 0;
                  governor.lastErrSig = "";
                  governor.sameErrRepeats = 0;
                  governor.lastOutHash = 0n;
                  governor.sameOutRepeats = 0;
                }

                toolResult = failed
                  ? `FAILED (exit_code: ${exitCode}).\n${cwdLine}\nstderr: ${stderr || "(empty)"}\nstdout: ${stdout || "(empty)"}${hint ? ("\n\n" + hint) : ""}\n⚠ The command failed. Diagnose the error above and call exec again with the fix. Do NOT continue to the next step until this succeeds.`
                  : `OK (exit_code: 0)\n${cwdLine}\nstdout: ${stdout || "(empty)"}`;

                if (stdout) display += "\n" + stdout;
                if (stderr) display += "\nstderr: " + stderr;
                display += "\n```\nexit: " + exitCode;
              } catch (execErr) {
                noteCmd(k, true, cmdSig(execErr.message || "", ""));
                toolResult = `error: ${execErr.message}`;
                display += "\nerror: " + execErr.message + "\n```";
              }
              flush();

              messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
              continue;
            }

            if (toolName === "write_file") {
              let args;
              try { args = JSON.parse(tc.function.arguments || "{}"); } catch (_) { args = {}; }
              const path0 = normalizePathSep(String(args.path || args.file_path || args.filename || ""));
              const content = String(args.content || args.text || "");

              const unsafePath =
                !path0 ||
                /^\//.test(path0) ||
                /^[a-zA-Z]:[\\/]/.test(path0) ||
                /(^|\/)\.\.(\/|$)/.test(path0);

              const win = isWindowsHost();
              const relShown = String(path0 || "(missing path)");
              const shown = truncTool(content, 2600);
              display += (display ? "\n\n" : "") + "```text " + relShown + "\n" + (shown || "(empty)") + "\n```";
              flush();

              let toolResult;
              try {
                if (unsafePath) {
                  toolResult = "error: unsafe path (must be relative, no '..', no drive letters)";
                  messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
                  continue;
                }
                if (!content) {
                  toolResult = "error: write_file content is empty";
                  messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
                  continue;
                }

                const cwdUsed = cwdNow();
                const base = cwdUsed ? normalizePathSep(String(cwdUsed)).replace(/\/+$/g, "") : "";
                const fullPath = base ? (base + "/" + path0.replace(/^\/+/g, "")) : path0;

                if (config.requireEditApproval) {
                  const q = await postJson("/api/queue_edit", {
                    action: "write_file",
                    path: fullPath,
                    content,
                  }, ac.signal);
                  const aid = String((q && q.approval_id) || "");
                  refreshPendingEdits();
                  toolResult = aid
                    ? `Awaiting approval via /api/approve_edit\napproval_id: ${aid}`
                    : "Awaiting approval via /api/approve_edit";
                  messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
                  awaitingApproval = true;
                  break;
                }

                const wr = await postJson("/api/write_file", { path: fullPath, content }, ac.signal);
                toolResult = `OK write_file\nbytes_written: ${wr && wr.bytes_written != null ? wr.bytes_written : content.length}`;
                messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
              } catch (e2) {
                toolResult = `error: ${prettyErr(e2)}`;
                messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
              }
              flush();
              if (awaitingApproval) break;
              continue;
            }

            // Unknown tool — ignore, but keep the model informed.
            messages.push({ role: "tool", tool_call_id: tc.id, content: `error: unknown tool: ${toolName}` });
          }
          if (awaitingApproval) break;
        } else {
          // finish_reason === "stop" — if the model didn't produce tool calls, try implied scripts.
          const implied = extractImpliedExecScripts(asstText);
          if (!implied.length) {
            // Goal delta check: the model may "stop" even though the repo/task isn't actually complete.
            // For repo-scaffolding style prompts, run ONE lightweight sanity probe and continue if goals are missing.
            const canExec = !!(status && status.features && status.features.exec);
            if (WANTS_REPO_GOAL && goalChecks < 1 && canExec) {
              goalChecks++;

              const win = isWindowsHost();
              const fenceLang = win ? "powershell" : "bash";
              const prompt = win ? "PS> " : "$ ";
              const probeCmd = win
                ? [
                    "$inRepo = Test-Path -LiteralPath '.git'",
                    "$head = ''",
                    "try { $head = (git rev-parse HEAD 2>$null).Trim() } catch { $head = '' }",
                    "$readme = Test-Path -LiteralPath 'README.md'",
                    "Write-Output ('in_repo=' + $inRepo)",
                    "Write-Output ('head=' + $head)",
                    "Write-Output ('readme=' + $readme)",
                  ].join('; ')
                : [
                    "test -d .git && echo in_repo=true || echo in_repo=false",
                    "h=$(git rev-parse HEAD 2>/dev/null || true); echo head=$h",
                    "test -f README.md && echo readme=true || echo readme=false",
                  ].join("; ");

              display += (display ? "\n\n" : "") + "```" + fenceLang + "\n" + prompt + "# [auto] goal check (repo)\n" + probeCmd;
              flush("\n```");

              try {
                const cwdUsed = cwdNow();
                const execCmd = wrapExecWithPwd(probeCmd);
                const execRes = await postJson("/api/exec", { command: execCmd, cwd: cwdUsed }, ac.signal);
                const parsed = stripPwdMarker(execRes.stdout);
                const breach = sandboxBreachReason(parsed.pwd);
                maybeUpdateWorkdirFromPwd(parsed.pwd);
                const stdout = String(parsed.stdout || "").trim();

                const mRepo = stdout.match(/(^|\n)in_repo=(true|false)\b/i);
                const mHead = stdout.match(/(^|\n)head=([0-9a-f]{6,40})\b/i);
                const mReadme = stdout.match(/(^|\n)readme=(true|false)\b/i);

                const inRepo = !!(mRepo && String(mRepo[2] || "").toLowerCase() === "true");
                const hasHead = !!(mHead && mHead[2]);
                const hasReadme = !!(mReadme && String(mReadme[2] || "").toLowerCase() === "true");

                const missing = [];
                if (!inRepo) missing.push("git init (no .git)");
                if (inRepo && !hasHead) missing.push("initial commit");
                if (!hasReadme) missing.push("README.md");

                display += (stdout ? ("\n" + truncTool(stdout, TRUNC_STDOUT)) : "\n(stdout empty)") + "\n```";
                flush();

                if (breach) {
                  messages.push({
                    role: "user",
                    content: [
                      "[sandbox_breach]",
                      "A command ended outside tool_root. This is blocked to prevent repo-root modification accidents.",
                      breach,
                      "Fix: re-run under tool_root; avoid `cd ..` / absolute paths. Verify `pwd` stays under tool_root.",
                    ].join("\n"),
                  });
                  continue;
                }

                if (missing.length) {
                  messages.push({
                    role: "user",
                    content: [
                      "[goal_check]",
                      "The task is NOT complete yet.",
                      "Missing: " + missing.join(", "),
                      "Fix it by using exec/write_file. Do NOT stop until the goals are satisfied.",
                    ].join("\n"),
                  });
                  continue;
                }
              } catch (e2) {
                display += "\nerror: " + prettyErr(e2) + "\n```";
                flush();
                // If probe fails, don't hard-block completion; fall through to break.
              }
            }
            break;
          }
          for (const im of implied) {
            if (ac.signal.aborted) break;
            const command = stripShellTranscript(String(im.script || ""));
            if (!command) continue;
            const commandToRun = normalizeExecScript(im.langHint, command);
            const k = cmdKey(commandToRun);
            const repeatBlock = blockedByRepeatFailure(k);
            const danger = dangerousCommandReason(commandToRun);
            const win = isWindowsHost();
            const fenceLang = win ? "powershell" : "bash";
            const prompt = win ? "PS> " : "$ ";
            const shown = command.split("\n").map((l, i) => (i === 0 ? prompt : "    ") + l).join("\n");
            display += (display ? "\n\n" : "") + "```" + fenceLang + "\n" + shown;
            flush("\n```");

            let resultText;
            try {
              if (repeatBlock) {
                resultText = `error: blocked repeated failing command (${repeatBlock}). You MUST choose a different command/strategy.`;
                display += `\n(blocked: ${repeatBlock})\n\`\`\`\nexit: -1`;
                flush();
                messages.push({ role: "user", content: `[exec blocked repeated-failure]\n${repeatBlock}\ncommand:\n${commandToRun}` });
                break;
              }
              if (danger) {
                resultText = `error: blocked dangerous command (${danger}). Ask the user to run it manually if truly intended.`;
                display += `\n(blocked: ${danger})\n\`\`\`\nexit: -1`;
                flush();
                messages.push({ role: "user", content: `[exec blocked]\nreason: ${danger}\ncommand:\n${commandToRun}` });
                break;
              }
              if (config.requireCommandApproval && !window.confirm("Run command?\n\n" + commandToRun)) {
                resultText = "error: command rejected by user";
                display += "\n(rejected)\n```\nexit: -1";
                flush();
                messages.push({ role: "user", content: `[exec rejected]\ncommand:\n${commandToRun}` });
                break;
              }

              const cwdUsed = cwdNow();
              const execCmd = wrapExecWithPwd(commandToRun);
              const execRes = await postJson("/api/exec", { command: execCmd, cwd: cwdUsed }, ac.signal);
              const parsed = stripPwdMarker(execRes.stdout);
              const breach = sandboxBreachReason(parsed.pwd);
              maybeUpdateWorkdirFromPwd(parsed.pwd);
              const stdout = truncTool(parsed.stdout, TRUNC_STDOUT);
              const stderr = truncTool(execRes.stderr, TRUNC_STDERR);
              const exitCode = execRes.exit_code;
              const looksHardError = (t) => /(^|\n)\s*(fatal:|error:|exception|traceback)\b/i.test(String(t || ""));
              const failed = exitCode !== 0 || looksHardError(stderr) || !!breach;
              noteCmd(k, failed, cmdSig(stderr, stdout));
              const hintGit = failed ? gitRepoHint(stderr) : "";
              const hintGov = failed ? deriveGovernorHint(stderr, stdout) : "";
              const hintSandbox = breach ? [
                "SANDBOX BREACH: command ended outside tool_root.",
                breach,
                "Fix: re-run under tool_root; avoid `cd ..` / absolute paths. Verify `pwd` stays under tool_root.",
              ].join("\n") : "";
              const hint = [hintGit, hintGov, hintSandbox].filter(Boolean).join("\n\n");

              const cwdUsedLabel = cwdUsed ? String(cwdUsed) : "(workspace root)";
              const cwdAfter = cwdNow();
              const cwdAfterLabel = cwdAfter ? String(cwdAfter) : "(workspace root)";
              const cwdLine = cwdUsedLabel === cwdAfterLabel
                ? `cwd: ${cwdUsedLabel}`
                : `cwd: ${cwdUsedLabel}\ncwd_after: ${cwdAfterLabel}`;

              if (failed) {
                governor.consecutiveFailures++;
                const sig0 = cmdSig(stderr, stdout);
                const sig = sig0 || (breach ? String(breach).slice(0, 180) : "");
                if (sig && sig === governor.lastErrSig) governor.sameErrRepeats++;
                else { governor.lastErrSig = sig; governor.sameErrRepeats = 1; }
                const outHash = fnv1a64(String(stderr || "") + "\n" + String(stdout || ""));
                if (outHash === governor.lastOutHash) governor.sameOutRepeats++;
                else { governor.lastOutHash = outHash; governor.sameOutRepeats = 1; }

                if (governor.sameErrRepeats >= 2 || governor.sameOutRepeats >= 2 || governor.consecutiveFailures >= 3) {
                  const stuck = [
                    "You are stuck in a failure loop.",
                    governor.lastErrSig ? ("last_error_signature: " + governor.lastErrSig) : "",
                    "STOP repeating the same approach. Change strategy.",
                    hintGov || hintGit || "",
                    "First verify cwd/tool_root, then pick a different command.",
                  ].filter(Boolean).join("\n");
                  governor.pendingHint = breach ? (hintSandbox + "\n\n" + stuck) : stuck;
                } else if (breach) {
                  governor.pendingHint = hintSandbox;
                }
              } else {
                governor.consecutiveFailures = 0;
                governor.lastErrSig = "";
                governor.sameErrRepeats = 0;
                governor.lastOutHash = 0n;
                governor.sameOutRepeats = 0;
              }

              resultText = failed
                ? `FAILED (exit_code: ${exitCode}).\n${cwdLine}\nstderr: ${stderr || "(empty)"}\nstdout: ${stdout || "(empty)"}${hint ? ("\n\n" + hint) : ""}`
                : `OK (exit_code: 0)\n${cwdLine}\nstdout: ${stdout || "(empty)"}`;

              if (stdout) display += "\n" + stdout;
              if (stderr) display += "\nstderr: " + stderr;
              display += "\n```\nexit: " + exitCode;
              flush();

              messages.push({
                role: "user",
                content: [
                  "[exec result]",
                  "command:",
                  commandToRun,
                  resultText,
                  "Continue by outputting the NEXT command(s) as ONE code block (or use exec tool calls if supported).",
                ].join("\n"),
              });
            } catch (execErr) {
              noteCmd(k, true, cmdSig(execErr.message || "", ""));
              resultText = `error: ${execErr.message}`;
              display += "\nerror: " + execErr.message + "\n```";
              flush();
              messages.push({ role: "user", content: `[exec error]\n${execErr.message}` });
              break;
            }
          }
          break; // implied exec done — don't loop back; prevents "complete" notification spam
        }
      }

      finishStreaming(threadId, asstMsgId);
    };

      const sendCoder = async (overrideText) => {
        if (sendingCoder) return;
        const raw = overrideText != null ? String(overrideText) : String(coderInput || "");
        const text = raw.trim();
        if (!text) return;

        // Local slash commands (UI-side). These do not call the model.
        // They are useful when using the Mistral VIBE CLI provider (mistral-cli).
        if (overrideText == null && text.startsWith("/")) {
          const parts = text.split(/\s+/);
          const cmd = String(parts[0] || "").trim().toLowerCase();
          const arg = parts.slice(1).join(" ").trim();
          if (cmd === "/agent") {
            setConfig((c) => ({ ...c, mistralCliAgent: arg }));
            const msg = { id: uid(), pane: "coder", role: "assistant", content: `[OBSTRAL] vibe agent = ${arg || "(default)"}`, ts: Date.now() };
            setThreadState((s) => ({
              ...s,
              threads: s.threads.map((t) => (t.id === activeThread.id ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), msg] } : t)),
            }));
            setCoderInput("");
            return;
          }
          if (cmd === "/turns") {
            const n = String(arg || "").trim();
            setConfig((c) => ({ ...c, mistralCliMaxTurns: n }));
            const msg = { id: uid(), pane: "coder", role: "assistant", content: `[OBSTRAL] vibe max_turns = ${n || "(default)"}`, ts: Date.now() };
            setThreadState((s) => ({
              ...s,
              threads: s.threads.map((t) => (t.id === activeThread.id ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), msg] } : t)),
            }));
            setCoderInput("");
            return;
          }
          if (cmd === "/scaffold") {
            const canExec = !!(status && status.features && status.features.exec);
            if (!canExec) {
              const msg = { id: uid(), pane: "coder", role: "assistant", content: "[OBSTRAL] /scaffold requires /api/exec", ts: Date.now() };
              setThreadState((s) => ({
                ...s,
                threads: s.threads.map((t) => (t.id === activeThread.id ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), msg] } : t)),
              }));
              setCoderInput("");
              return;
            }

            const name0 = String(arg || "").trim();
            const safe = name0
              .replace(/[\\/:*?\"<>|]/g, "_")
              .replace(/\s+/g, "-")
              .replace(/-+/g, "-")
              .replace(/^\.+/g, "")
              .replace(/_+/g, "_")
              .trim();
            if (!safe) {
              const msg = { id: uid(), pane: "coder", role: "assistant", content: "[OBSTRAL] usage: /scaffold <repo-name>", ts: Date.now() };
              setThreadState((s) => ({
                ...s,
                threads: s.threads.map((t) => (t.id === activeThread.id ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), msg] } : t)),
              }));
              setCoderInput("");
              return;
            }

            const baseCwd = resolvedThreadRoot(config.toolRoot, activeThread && activeThread.id);
            const readmeEn = [
              `# ${safe}`,
              "",
              "Generated by OBSTRAL /scaffold.",
              "",
              "This repository is a minimal template generated by OBSTRAL.",
              "",
              "Readme translations:",
              "- Japanese: README.ja.md",
              "- French: README.fr.md",
              "",
            ];
            const readmeJa = [
              `# ${safe}`,
              "",
              "OBSTRAL /scaffold により生成されました。",
              "",
              "このリポジトリはOBSTRALで自動生成された最小テンプレートです。",
              "",
              "README翻訳:",
              "- English: README.md",
              "- French: README.fr.md",
              "",
            ];
            const readmeFr = [
              `# ${safe}`,
              "",
              "Généré par OBSTRAL /scaffold.",
              "",
              "Ce dépôt est un modèle minimal généré par OBSTRAL.",
              "",
              "Traductions du README :",
              "- Anglais : README.md",
              "- Japonais : README.ja.md",
              "",
            ];
            const readmeEnPs = "@(" + readmeEn.map(psSingleQuote).join(",") + ") | Set-Content -LiteralPath 'README.md' -Encoding UTF8";
            const readmeJaPs = "@(" + readmeJa.map(psSingleQuote).join(",") + ") | Set-Content -LiteralPath 'README.ja.md' -Encoding UTF8";
            const readmeFrPs = "@(" + readmeFr.map(psSingleQuote).join(",") + ") | Set-Content -LiteralPath 'README.fr.md' -Encoding UTF8";
            const gitignoreLines = [
              "# OBSTRAL scaffold",
              ".DS_Store",
              "node_modules/",
              "dist/",
              "target/",
              ".venv/",
              "__pycache__/",
              "*.log",
              "",
            ];
            const gitignorePs = "@(" + gitignoreLines.map(psSingleQuote).join(",") + ") | Set-Content -LiteralPath '.gitignore' -Encoding UTF8";
            const cmdPs = [
              "$ErrorActionPreference = 'Stop'",
              `New-Item -ItemType Directory -Force -Path ${psSingleQuote(safe)} | Out-Null`,
              `Set-Location ${psSingleQuote(safe)}`,
              "New-Item -ItemType Directory -Force -Path 'src' | Out-Null",
              "New-Item -ItemType Directory -Force -Path 'docs' | Out-Null",
              readmeEnPs,
              readmeJaPs,
              readmeFrPs,
              gitignorePs,
              "git init | Out-Null",
              "$n = (git config user.name); if (-not $n) { git config user.name 'OBSTRAL' }",
              "$e = (git config user.email); if (-not $e) { git config user.email 'obstral@local' }",
              "git add README.md README.ja.md README.fr.md .gitignore | Out-Null",
              "git commit -m 'Initial commit' | Out-Null",
            ].join("; ");

            try {
              const res = await postJson("/api/exec", { command: cmdPs, cwd: baseCwd });
              const ok = (res && typeof res.exit_code === "number") ? res.exit_code === 0 : false;
              const out = ok
                ? `[OBSTRAL] scaffolded repo: ${safe} (cwd=${String(res.cwd || baseCwd || "")})`
                : `[OBSTRAL] scaffold failed (exit_code=${res.exit_code}). stderr: ${String(res.stderr || "(empty)")}`;
              const msg = { id: uid(), pane: "coder", role: "assistant", content: out, ts: Date.now() };
              setThreadState((s) => ({
                ...s,
                threads: s.threads.map((t) => (
                  t.id === activeThread.id
                    ? { ...t, updatedAt: Date.now(), workdir: ok ? safe : (t.workdir || ""), messages: [...(t.messages || []), msg] }
                    : t
                )),
              }));
            } catch (err) {
              const msg = { id: uid(), pane: "coder", role: "assistant", content: `[OBSTRAL] scaffold error: ${prettyErr(err)}`, ts: Date.now() };
              setThreadState((s) => ({
                ...s,
                threads: s.threads.map((t) => (t.id === activeThread.id ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), msg] } : t)),
              }));
            }
            setCoderInput("");
            return;
          }
        }

        const coderCfg = String(config.mode || "").trim() === "Observer" ? { ...config, mode: "VIBE" } : config;
        if (String(config.mode || "").trim() === "Observer") {
          setConfig((c) => ({ ...c, mode: "VIBE" }));
        }

        const wantsMaterial = /(?:repo|repository|scaffold|bootstrap|init|setup|create|generate|implement|install|build|test|run|git|winget|bash|powershell|cmd|command|readme|license|requirements|folder|directory|depot|dépôt|referentiel|référentiel|projet|dossier|répertoire|repertoire|fichier|commande|créer|cree|crée|générer|generer|implément|implementer|exécuter|execute|installer|リポ|リポジトリ|雛形|ひな形|フォルダ|ふぉるだ|ディレクトリ|でぃれくとり|プロジェクト|実装|ファイル|作成|作ろ|作って|生成|追加|組み込|編集|更新|書いて|書き直して|コマンド|実行|インストール|セットアップ|ビルド|テスト|自分で|じぶんで|やって|やってみて)/i.test(text);
        const useCode = modeUsesCode(coderCfg.mode) || wantsMaterial;
        const resolvedProvider = useCode ? (String(coderCfg.codeProvider || "").trim() || coderCfg.provider) : coderCfg.provider;
        const resolvedBaseUrl = useCode ? (String(coderCfg.codeBaseUrl || "").trim() || coderCfg.baseUrl) : coderCfg.baseUrl;
        const resolvedModel = useCode ? (coderCfg.codeModel || coderCfg.model) : (coderCfg.chatModel || coderCfg.model);
        const resolvedKey = useCode
          ? (String(codeApiKey || "").trim() || String(chatApiKey || "").trim() || String(observerApiKey || "").trim())
          : (String(chatApiKey || "").trim() || String(codeApiKey || "").trim() || String(observerApiKey || "").trim());
        const reqCfg = {
          ...coderCfg,
          provider: resolvedProvider,
          baseUrl: resolvedBaseUrl,
          model: resolvedModel,
          chatModel: resolvedModel,
          codeModel: resolvedModel,
        };

        const threadId = activeThread.id;
        const history = paneMessages("coder").map((m) => ({ role: m.role, content: m.content }));

      const userMsg = { id: uid(), pane: "coder", role: "user", content: text, ts: Date.now() };
      const asstMsg = { id: uid(), pane: "coder", role: "assistant", content: "", ts: Date.now(), streaming: true };

      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) =>
          t.id === threadId ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), userMsg, asstMsg] } : t
        ),
      }));
      ensureTitle(threadId, text);
      setCoderInput("");
      setSendingCoder(true);
      requestAnimationFrame(() => scrollBottom(coderBodyRef));

      const toolRootResolved = resolvedCwd(config.toolRoot, threadId, activeThread && activeThread.workdir);
      const reqCfg2 = toolRootResolved ? { ...reqCfg, toolRoot: toolRootResolved } : reqCfg;
      const reqBody = buildReq(reqCfg2, resolvedKey, history, text, diff);
      reqBody.lang = lang;
      reqBody.force_tools = !!(useCode || wantsMaterial);
      const ac = new AbortController();
      abortCoderRef.current = ac;

      try {
          const supportsTools = resolvedProvider === "openai-compatible" || resolvedProvider === "mistral" || resolvedProvider === "openai";
          const serverChatTools = !!(status && status.features && status.features.chat_tools);
          if ((config.forceAgent || wantsMaterial) && supportsTools && serverChatTools) {
            await runCoderAgentic(text, threadId, asstMsg.id, reqCfg, resolvedKey, history, ac, activeThread && activeThread.workdir);
          } else if (config.stream) {
          await streamChat(
            reqBody,
            (evt) => {
              if (!evt) return;
              if (evt.event === "delta") {
                const j = safeJsonParse(evt.data || "{}", {});
                if (j && j.delta) appendDelta(threadId, asstMsg.id, j.delta, coderBodyRef);
              } else if (evt.event === "error") {
                const j = safeJsonParse(evt.data || "{}", {});
                throw new Error(j.error || tr(lang, "error"));
              }
            },
            ac.signal
          );
          finishStreaming(threadId, asstMsg.id);
        } else {
          const j = await postJson("/api/chat", reqBody, ac.signal);
          setMsg(threadId, asstMsg.id, String((j && j.content) || ""), coderBodyRef);
        }
      } catch (err) {
        const msg = prettyErr(err);
        if (config.stream && !ac.signal.aborted) {
          try {
            const j = await postJson("/api/chat", reqBody, ac.signal);
            setMsg(threadId, asstMsg.id, String((j && j.content) || ""), coderBodyRef);
          } catch (err2) {
            const msg2 = prettyErr(err2);
            setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg2}`, coderBodyRef);
          }
        } else if (ac.signal.aborted) {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "stop")}]`, coderBodyRef);
        } else {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg}`, coderBodyRef);
        }
      } finally {
        setSendingCoder(false);
        abortCoderRef.current = null;
        refreshPendingEdits();
      }
    };

    const sendObserver = async (overrideText) => {
      if (sendingObserver) return;
      const text = overrideText != null ? String(overrideText) : String(observerInput || "").trim();
      if (!text) return;

      const threadId = activeThread.id;
      const history = paneMessages("observer").map((m) => ({ role: m.role, content: m.content }));
      const prevObserverAssts = (() => {
        const out = [];
        for (let i = history.length - 1; i >= 0 && out.length < 4; i--) {
          const m = history[i];
          if (m && m.role === "assistant") out.push(String(m.content || ""));
        }
        return out;
      })();

      const userMsg = { id: uid(), pane: "observer", role: "user", content: text, ts: Date.now() };
      const asstMsg = { id: uid(), pane: "observer", role: "assistant", content: "", ts: Date.now(), streaming: true };

      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) =>
          t.id === threadId ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), userMsg, asstMsg] } : t
        ),
      }));
      if (overrideText == null) setObserverInput("");
      setSendingObserver(true);
      requestAnimationFrame(() => scrollBottom(observerBodyRef));

      // Keep Observer in-character: do not inherit coder's CoT/autonomy formatting.
      const obsProvider = String(config.observerProvider || "").trim() || config.provider;
      const obsBaseUrl = String(config.observerBaseUrl || "").trim() || config.baseUrl;
      const obsModel = String(config.observerModel || "").trim() || (config.chatModel || config.model);
      const obsKey = String(observerApiKey || "").trim() || String(chatApiKey || "").trim() || String(codeApiKey || "").trim();
      const obsCfg = {
        ...config,
        mode: config.observerMode,
        persona: config.observerPersona,
        cot: "off",
        autonomy: "off",
        provider: obsProvider,
        baseUrl: obsBaseUrl,
        model: obsModel,
        chatModel: obsModel,
        codeModel: obsModel,
      };
      const intensity0 = String(config.observerIntensity || "critical").trim().toLowerCase();
      const intensity = intensity0 === "polite" || intensity0 === "critical" || intensity0 === "brutal" ? intensity0 : "critical";
      let intensityInstr = "";
      if (intensity === "polite") {
        intensityInstr = [
          "Intensity: polite. Be constructive and encouraging.",
          "Still flag concrete issues across all five dimensions (correctness/security/reliability/performance/maintainability).",
          "Every proposal must include a specific, actionable to_coder message.",
          "Anti-loop: flag NEW issues only. If nothing new, summarise still-open items from prior critiques as [OPEN].",
        ].join("\n");
      } else if (intensity === "brutal") {
        intensityInstr = [
          "Intensity: brutal. Assume this code ships to 10,000 users at midnight. Find every failure mode.",
          "Required: identify at least TWO failure modes — one correctness/data bug and one operational risk (monitoring, rollback, config drift).",
          "Required: every proposal must include a specific to_coder message and realistic impact estimate.",
          "Forbidden: 'looks good', 'nice work', 'could consider'. Only concrete flaws with concrete fixes.",
          "Anti-loop: if a prior issue is still unresolved, escalate its score by 10 and mark [ESCALATED]. New findings only otherwise.",
        ].join("\n");
      } else {
        intensityInstr = [
          "Intensity: critical. Treat this as a pre-merge review for a production service.",
          "Required: identify at least ONE concrete bug, security risk, or architectural weakness with a specific to_coder message.",
          "Check for: missing input validation, unhandled errors, hardcoded values, and missing test coverage.",
          "Anti-loop: if you raised this issue before, mark it [UNRESOLVED] and move on. No new signal → reply exactly: [Observer] No new critique. Loop detected.",
        ].join("\n");
      }
      const loopLine =
        loopInfo && loopInfo.depth > 0
          ? `ui_loop_detected: depth=${loopInfo.depth} sim=${Math.round((loopInfo.score || 0) * 100)}%`
          : "";
      const outLang = String(lang || "ja").trim().toLowerCase();
      const proposalKeysLine = "Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost).";
      const langLine = outLang === "fr"
        ? [
            "Language: French.",
            "Langue: français.",
            "Write the critique in French.",
            "Écris la critique en français.",
            "Do not write in English.",
            "N'écris pas en anglais.",
            proposalKeysLine,
            "Garde les clés du bloc proposals en anglais (title/to_coder/severity/score/phase/impact/cost).",
          ].join("\n")
        : outLang === "en"
          ? [
              "Language: English.",
              "Write the critique in English.",
              proposalKeysLine,
            ].join("\n")
          : [
              "Language: Japanese.",
              "言語: 日本語。",
              "Write the critique in Japanese.",
              "批評は日本語で書いてください。",
              "Do not write in English.",
              "英語で書かないでください。",
              proposalKeysLine,
              "proposalsブロックのキー(title/to_coder/severity/score/phase/impact/cost)は英語のままにしてください。",
            ].join("\n");

      // Lightweight proposal memory: provide the Observer with a compact list of recent proposals and approvals
      // so it can avoid repeating the same template critique and can mark items UNRESOLVED/ESCALATED accurately.
      const priorProposalsSummary = (() => {
        try {
          const normTitle = (s) => String(s || "").trim().toLowerCase().replace(/\s+/g, " ");
          const approved = new Set();
          const coderMsgs = paneMessages("coder");
          for (const m of coderMsgs || []) {
            if (!m || m.role !== "user") continue;
            const t = String(m.content || "");
            if (!t.startsWith("[Observer proposal approved]")) continue;
            const mm = t.match(/^Title:\s*(.+)$/im);
            const title = mm ? mm[1] : "";
            const k = normTitle(title);
            if (k) approved.add(k);
          }

          const obsMsgs = paneMessages("observer");
          const map = new Map(); // normTitle -> proposal
          let scanned = 0;
          for (let i = (obsMsgs || []).length - 1; i >= 0 && scanned < 12; i--) {
            const m = obsMsgs[i];
            if (!m || m.role !== "assistant" || m.streaming) continue;
            scanned++;
            const props = parseProposals(String(m.content || ""));
            for (const p of props || []) {
              const k = normTitle(p.title);
              if (!k) continue;
              const prev = map.get(k);
              // Keep the latest/highest-score version (they can differ per turn).
              if (!prev || (p.score || 0) >= (prev.score || 0)) {
                map.set(k, { ...p, _approved: approved.has(k) });
              }
            }
          }

          const arr = Array.from(map.values())
            .sort((a, b) => (b.score || 50) - (a.score || 50))
            .slice(0, 6);

          if (!arr.length) return "";
          const lines = arr.map((p) => {
            const tag = p._approved ? "[APPROVED]" : "[OPEN]";
            const sev = String(p.severity || "info");
            const sc = typeof p.score === "number" ? p.score : 50;
            const ph = String(p.phase || "any");
            return `- ${tag} ${p.title} (sev:${sev} score:${sc} phase:${ph})`;
          });
          return ["prior_proposals:", ...lines].join("\n");
        } catch (_) {
          return "";
        }
      })();
      const observerBridge = [
        "[Observer bridge]",
        langLine,
        `observer_intensity: ${intensity}`,
        loopLine,
        intensityInstr,
        "Meta ops (optional): if you propose updating OBSTRAL runtime prompts, start to_coder with one of: META_SET_CODER:, META_APPEND_CODER:, META_SET_OBSERVER:, META_APPEND_OBSERVER:.",
        "Review the coder's artifacts below. Check each dimension: CORRECTNESS, SECURITY, RELIABILITY, PERFORMANCE, MAINTAINABILITY.",
        "Code citation: for every warn/crit proposal, add a quote: field containing an exact function name,",
        "  variable name, or ≤40-char code snippet from the coder's output. Use n/a only if no code is visible.",
        priorProposalsSummary,
        "Follow-through: scan for prior proposals in the conversation.",
        "  If addressed by the Coder: mark status: addressed.",
        "  If still unresolved: mark status: [UNRESOLVED] and add +10 to the score.",
        "After proposals, always append BOTH additional blocks:",
        "--- critical_path ---",
        "<ONE sentence: the single issue that, if unaddressed, makes all other improvements pointless. Or 'none' if no blockers.>",
        "--- health ---",
        "score: <0-100 integer: 0=won't run, 50=works-but-risky, 100=shippable>",
        "rationale: <one sentence explaining the score>",
      ].filter(Boolean).join("\n");
      const sendText = config.includeCoderContext
        ? (text + "\n\n" + observerBridge + "\n\n" + coderContextPacket())
        : (text + "\n\n" + observerBridge);
      const toolRootResolved = resolvedCwd(config.toolRoot, threadId, activeThread && activeThread.workdir);
      const obsCfg2 = toolRootResolved ? { ...obsCfg, toolRoot: toolRootResolved } : obsCfg;
      const reqBody = buildReq(obsCfg2, obsKey, history, sendText, diff);
      reqBody.lang = lang;
      reqBody.force_tools = false;
      const ac = new AbortController();
      abortObserverRef.current = ac;

      try {
        // One-shot language retry: if the response ignores the requested language (ja/fr),
        // retry once with an explicit rewrite instruction.
         const countRe = (re, s) => (String(s || "").match(re) || []).length;
         const stripFencesForLang = (s) => {
           const raw = String(s || "").replace(/\r\n/g, "\n");
           const out = [];
           let inFence = false;
           for (const line of raw.split("\n")) {
             if (/^\s*```/.test(line)) { inFence = !inFence; continue; }
             if (inFence) continue;
             out.push(line);
           }
           return out.join("\n");
          };
          const looksJapanese = (s) => {
            const x = stripFencesForLang(stripObserverMeta(s));
            const jp = countRe(/[\u3040-\u30ff\u3400-\u4dbf\u4e00-\u9fff]/g, x);
            const lat = countRe(/[A-Za-z]/g, x);
            if (jp < 8) return false;
            if (lat <= 0) return true;
           // Allow some English tokens (code, keys) but avoid "mostly English with a few JP chars".
           return lat <= jp * 2;
          };
         const looksFrench = (s) => {
          const x = stripFencesForLang(stripObserverMeta(s));
           const a = countRe(/[\u00C0-\u017F]/g, x);
           const fr = countRe(/\b(le|la|les|des|du|de|pour|avec|sans|est|sont|pas|mais|donc|sur|dans|vous|tu|je|nous|votre)\b/gi, x);
           const en = countRe(/\b(the|and|you|your|should|this|that|with|for|not|are|is|was|were|will|can|cannot|do|does)\b/gi, x);
           if (a > 0 && fr >= 1) return true;
          return fr > en + 1;
        };
        const skippable = (s) => {
          const t = String(s || "").trim();
          if (!t) return true;
          if (t.startsWith("[Observer]")) return true;
          if (/^\[(error|erreur|エラー)\]/i.test(t)) return true;
          if (t.startsWith("[" + tr(lang, "error") + "]")) return true;
          if (t.startsWith("[" + tr(lang, "stop") + "]")) return true;
          return false;
        };
        const needsLangRetry = (expected, content) => {
          if (skippable(content)) return false;
          if (expected === "ja") return !looksJapanese(content);
          if (expected === "fr") return !looksFrench(content);
          return false;
        };

        let finalText = "";
        if (config.stream) {
          let acc = "";
          await streamChat(
            reqBody,
            (evt) => {
              if (!evt) return;
              if (evt.event === "delta") {
                const j = safeJsonParse(evt.data || "{}", {});
                if (j && j.delta) {
                  acc += String(j.delta);
                  appendDelta(threadId, asstMsg.id, j.delta, observerBodyRef);
                }
              } else if (evt.event === "error") {
                const j = safeJsonParse(evt.data || "{}", {});
                throw new Error(j.error || tr(lang, "error"));
              }
            },
            ac.signal
          );
          finishStreaming(threadId, asstMsg.id);
          finalText = acc;
        } else {
          const j = await postJson("/api/chat", reqBody, ac.signal);
          finalText = String((j && j.content) || "");
          setMsg(threadId, asstMsg.id, finalText, observerBodyRef);
        }

        if (!ac.signal.aborted && needsLangRetry(outLang, finalText)) {
          const msg =
            outLang === "fr"
              ? "Observer replied in the wrong language — retrying in French…"
              : outLang === "en"
                ? "Observer language mismatch — retrying…"
                : "Observerが指定言語で返していないため、再試行します…";
          showToast(msg, "info");

          const retryInstr =
            outLang === "fr"
              ? "LANGUAGE FIX: Rewrite the assistant's last message in French ONLY. Do not add new content. Output ONLY the rewritten text. Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost)."
              : outLang === "en"
                ? "LANGUAGE FIX: Rewrite the assistant's last message in English ONLY. Do not add new content. Output ONLY the rewritten text. Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost)."
                : "LANGUAGE FIX: Rewrite the assistant's last message in Japanese ONLY. Do not add new content. Output ONLY the rewritten text. Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost).";
          const extHistory = [
            ...history,
            { role: "user", content: sendText },
            { role: "assistant", content: finalText },
          ];
          const retryBody = buildReq(obsCfg, obsKey, extHistory, retryInstr + "\n\n" + observerBridge, diff);
          retryBody.lang = lang;
          retryBody.force_tools = false;
          // Make the rewrite more deterministic (language fix should not "drift").
          retryBody.temperature = 0.2;
          try {
            const j2 = await postJson("/api/chat", retryBody, ac.signal);
            const fixed = String((j2 && j2.content) || "");
            if (fixed) {
              finalText = fixed;
              setMsg(threadId, asstMsg.id, fixed, observerBodyRef);
            }
          } catch (_) {
            // If retry fails, keep the original response.
          }
        }

        // One-shot loop retry: if the Observer repeats itself with no new signal,
        // force a diff-style critique instead of reprinting the same template.
        if (!ac.signal.aborted && prevObserverAssts.length && !skippable(finalText)) {
          const maxSim = (() => {
            let s = 0;
            for (const prev of prevObserverAssts) s = Math.max(s, similarity(prev, finalText));
            return s;
          })();
          const titleJacc = (() => {
            try {
              const norm = (s) => String(s || "").trim().toLowerCase().replace(/\s+/g, " ");
              const a0 = prevObserverAssts[0] || "";
              const a = new Set(parseProposals(a0).map((p) => norm(p.title)).filter(Boolean));
              const b = new Set(parseProposals(finalText).map((p) => norm(p.title)).filter(Boolean));
              if (!a.size && !b.size) return 1;
              return jaccardSim(a, b);
            } catch (_) {
              return maxSim;
            }
          })();
          const loopish = maxSim >= 0.82 && titleJacc >= 0.75;
          if (loopish) {
            const msg =
              outLang === "fr"
                ? "Observer se répète — nouvelle tentative (diff-only)…"
                : outLang === "en"
                  ? "Observer repeated itself — retrying (diff-only)…"
                  : "Observerが同じ内容を繰り返しているため、差分批評で再試行します…";
            showToast(msg, "info");

            const loopFixInstr =
              outLang === "fr"
                ? "LOOP FIX: Ton dernier message répétait la même critique. Écris une NOUVELLE critique UNIQUEMENT à partir des informations NOUVELLES depuis ton message précédent. Ne répète pas les mêmes proposals. S'il n'y a pas de nouveau signal, réponds exactement: [Observer] No new critique. Loop detected."
                : outLang === "en"
                  ? "LOOP FIX: Your last message repeated the same critique. Write a NEW critique ONLY based on NEW information since your previous message. Do not restate the same proposals. If there is no new signal, reply exactly: [Observer] No new critique. Loop detected."
                  : "LOOP FIX: 直前の批評と内容がほぼ同一です。前回から増えた情報に基づく「新しい」批評だけを書いてください。同じ提案の焼き直しは禁止。新しい指摘が無い場合は、次の1行だけを厳密に出力: [Observer] No new critique. Loop detected.";
            const extHistory2 = [
              ...history,
              { role: "user", content: sendText },
              { role: "assistant", content: finalText },
            ];
            const retryBody2 = buildReq(obsCfg, obsKey, extHistory2, loopFixInstr + "\n\n" + observerBridge, diff);
            retryBody2.lang = lang;
            retryBody2.force_tools = false;
            retryBody2.temperature = 0.2;
            try {
              const j3 = await postJson("/api/chat", retryBody2, ac.signal);
              const fixed2 = String((j3 && j3.content) || "");
              if (fixed2) {
                finalText = fixed2;
                setMsg(threadId, asstMsg.id, fixed2, observerBodyRef);
              }
            } catch (_) {
              // Keep original if retry fails.
            }

            // If retry still loops, hard-stop it instead of spamming templates.
            const maxSim2 = (() => {
              let s = 0;
              for (const prev of prevObserverAssts) s = Math.max(s, similarity(prev, finalText));
              return s;
            })();
            if (maxSim2 >= 0.82 && !skippable(finalText)) {
              finalText = "[Observer] No new critique. Loop detected.";
              setMsg(threadId, asstMsg.id, finalText, observerBodyRef);
            }
          }
        }
      } catch (err) {
        const msg = prettyErr(err);
        if (config.stream && !ac.signal.aborted) {
          try {
            const j = await postJson("/api/chat", reqBody, ac.signal);
            setMsg(threadId, asstMsg.id, String((j && j.content) || ""), observerBodyRef);
          } catch (err2) {
            const msg2 = prettyErr(err2);
            setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg2}`, observerBodyRef);
          }
        } else if (ac.signal.aborted) {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "stop")}]`, observerBodyRef);
        } else {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg}`, observerBodyRef);
        }
      } finally {
        setSendingObserver(false);
        abortObserverRef.current = null;
        refreshPendingEdits();
      }
    };

    const sendProposalToCoder = (p) => {
      if (!p) return;
      const title = String(p.title || "").trim();
      const to = String(p.toCoder || "").trim();
      if (!to) return;
      const sev = String(p.severity || "info").trim();
      const steer = `[Observer proposal approved]\nTitle: ${title}\nSeverity: ${sev}\n\n${to}\n`;
      sendCoder(steer);
    };

    const upsertTasksForThread = (threadId, items) => {
      const incoming = Array.isArray(items) ? items : [];
      if (!incoming.length) return;
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => {
          if (t.id !== threadId) return t;
          const existing = Array.isArray(t.tasks) ? t.tasks : [];
          const seen = new Set(existing.map((x) => `${String(x.target || "coder")}|${normTitle(x.title || "")}`));
          const merged = [...existing];
          for (const it of incoming) {
            const k = `${String(it.target || "coder")}|${normTitle(it.title || "")}`;
            if (!k.endsWith("|") && !seen.has(k)) {
              seen.add(k);
              merged.push({
                id: it.id || uid(),
                target: it.target === "observer" ? "observer" : "coder",
                title: String(it.title || "").trim(),
                body: String(it.body || "").trim(),
                phase: String(it.phase || "any").trim().toLowerCase(),
                priority: typeof it.priority === "number" ? it.priority : null,
                status: String(it.status || "new").trim().toLowerCase() || "new",
                createdAt: typeof it.createdAt === "number" ? it.createdAt : Date.now(),
              });
            }
          }
          return { ...t, updatedAt: Date.now(), tasks: merged };
        }),
      }));
    };

    const patchTask = (threadId, taskId, patch) => {
      if (!taskId) return;
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => {
          if (t.id !== threadId) return t;
          const tasks = Array.isArray(t.tasks) ? t.tasks : [];
          return {
            ...t,
            updatedAt: Date.now(),
            tasks: tasks.map((x) => (x && x.id === taskId ? { ...x, ...patch } : x)),
          };
        }),
      }));
    };

    const dispatchTask = (task) => {
      if (!task || !activeThread) return;
      const threadId = activeThread.id;
      const title = String(task.title || "").trim();
      const body = String(task.body || "").trim();
      const id = String(task.id || "").trim();
      const payload = `[Task ${id}] ${title}\n\n${body}\n`;

      if (String(task.target || "") === "observer") {
        sendObserver(payload);
      } else {
        sendCoder(payload);
      }
      patchTask(threadId, id, { status: "sent" });
    };

    const markTaskDone = (task) => {
      if (!task || !activeThread) return;
      patchTask(activeThread.id, String(task.id || ""), { status: "done" });
    };

    const planTasksFromChat = async (userText, ctxText) => {
      if (!activeThread) return;
      if (planningTasks) return;

      const threadId = activeThread.id;
      const obsProvider = String(config.observerProvider || "").trim() || config.provider;
      const obsBaseUrl = String(config.observerBaseUrl || "").trim() || config.baseUrl;
      const obsModel = String(config.observerModel || "").trim() || (config.chatModel || config.model);
      const obsKey = String(observerApiKey || "").trim() || String(chatApiKey || "").trim() || String(codeApiKey || "").trim();
      if (!obsKey) return;

      setPlanningTasks(true);
      showToast(tr(lang, "planningTasks"), "info");

      const langName = (lang === "fr") ? "French" : (lang === "en") ? "English" : "Japanese";
      const routerPrompt = [
        "You are TaskRouter for OBSTRAL (behind-the-scenes).",
        "Return ONLY valid JSON. No markdown. No commentary.",
        "Schema: {\"tasks\":[{\"target\":\"coder|observer\",\"title\":\"...\",\"body\":\"...\",\"phase\":\"core|feature|polish|any\",\"priority\":0-100}]}",
        "Rules:",
        "- Keep it minimal: 2-6 tasks total.",
        "- coder tasks MUST be concrete (file edits, commands, checks).",
        "- observer tasks are audit/validation tasks (risks, checks, acceptance criteria).",
        `Write title/body in ${langName}.`,
        "",
        "User message:",
        String(userText || ""),
        ctxText ? ("\nContext:\n" + String(ctxText || "")) : "",
      ].join("\n");

      const routerCfg = {
        ...config,
        mode: "壁打ち",
        cot: "off",
        autonomy: "off",
        provider: obsProvider,
        baseUrl: obsBaseUrl,
        model: obsModel,
        chatModel: obsModel,
        codeModel: obsModel,
        persona: "default",
      };
      const req = buildReq(routerCfg, obsKey, [], routerPrompt, null);
      req.lang = lang;
      req.force_tools = false;
      req.temperature = 0.2;
      req.max_tokens = 900;

      try {
        const j = await postJson("/api/chat", req);
        const tasks = parseTasksJson(String((j && j.content) || ""));
        if (tasks.length) upsertTasksForThread(threadId, tasks);
      } catch (err) {
        showToast(prettyErr(err), "error");
      } finally {
        setPlanningTasks(false);
      }
    };

    const sendChat = async (overrideText) => {
      if (sendingChat) return;
      const raw = overrideText != null ? String(overrideText) : String(chatInput || "");
      const text = raw.trim();
      if (!text) return;
      if (!activeThread) return;
      const threadId = activeThread.id;
      const apiKey = String(chatApiKey || "").trim() || String(codeApiKey || "").trim() || String(observerApiKey || "").trim();
      const chatCfg = { ...config, mode: "会話", cot: "off", autonomy: "off", persona: config.chatPersona || "cheerful" };
      const userMsg = { id: uid(), pane: "chat", role: "user", content: text, ts: Date.now() };
      const asstMsg = { id: uid(), pane: "chat", role: "assistant", content: "", ts: Date.now(), streaming: true };
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) =>
          t.id === threadId ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), userMsg, asstMsg] } : t
        ),
      }));
      setChatInput("");
      setSendingChat(true);
      requestAnimationFrame(() => scrollBottom(chatBodyRef));
      const history = paneMessages("chat").filter((m) => !m.streaming && m.content).map((m) => ({ role: m.role, content: m.content }));
      const ctx = config.includeCoderContext ? coderContextPacket() : "";
      planTasksFromChat(text, ctx);
      const fullInput = ctx ? `${ctx}\n\n${text}` : text;
      const reqBody = buildReq(chatCfg, apiKey, history, fullInput, null);
      reqBody.lang = lang;
      reqBody.force_tools = false;
      const ac = new AbortController();
      abortChatRef.current = ac;
      try {
        if (config.stream) {
          await streamChat(reqBody, (evt) => {
            if (!evt) return;
            if (evt.event === "delta") {
              const j = safeJsonParse(evt.data || "{}", {});
              if (j && j.delta) appendDelta(threadId, asstMsg.id, j.delta, chatBodyRef);
            } else if (evt.event === "error") {
              const j = safeJsonParse(evt.data || "{}", {});
              throw new Error(j.error || tr(lang, "error"));
            }
          }, ac.signal);
          finishStreaming(threadId, asstMsg.id);
        } else {
          const j = await postJson("/api/chat", reqBody, ac.signal);
          setMsg(threadId, asstMsg.id, String((j && j.content) || ""), chatBodyRef);
        }
      } catch (err) {
        const msg2 = prettyErr(err);
        if (config.stream && !ac.signal.aborted) {
          try {
            const j = await postJson("/api/chat", reqBody, ac.signal);
            setMsg(threadId, asstMsg.id, String((j && j.content) || ""), chatBodyRef);
          } catch (_) {
            setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg2}`, chatBodyRef);
          }
        } else if (ac.signal.aborted) {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "stop")}]`, chatBodyRef);
        } else {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg2}`, chatBodyRef);
        }
        finishStreaming(threadId, asstMsg.id);
      } finally {
        setSendingChat(false);
        abortChatRef.current = null;
      }
    };

    const stopChat = () => {
      if (abortChatRef.current) abortChatRef.current.abort();
      abortChatRef.current = null;
      setSendingChat(false);
    };

    const applyMetaOp = async (metaOp) => {
      if (!metaOp) return;
      const supported = !!(status && status.features && status.features.meta_prompts);
      if (!supported) return;
      if (metaBusy) return;
      setMetaBusy(true);
      try {
        const j = await postJson("/api/meta_prompts", metaOp);
        refreshPendingEdits();
        const eid = j && j.approval_id ? String(j.approval_id) : "";
        if (eid) {
          const msg =
            String(lang || "").trim().toLowerCase() === "fr"
              ? `Mise à jour du meta prompt en attente d'approbation: ${eid}`
              : String(lang || "").trim().toLowerCase() === "en"
                ? `Meta prompt update queued for approval: ${eid}`
                : `メタプロンプト更新が承認待ちです: ${eid}`;
          showToast(msg, "success");
        }
      } catch (err) {
        showToast(prettyErr(err), "error");
      } finally {
        setMetaBusy(false);
      }
    };

      const coderMsgs = paneMessages("coder");
      const observerMsgs = paneMessages("observer");
      const filterMsgs = (msgs, q) => {
        const needle = String(q || "").trim().toLowerCase();
        if (!needle) return msgs;
        return (msgs || []).filter((m) => String(m && m.content ? m.content : "").toLowerCase().indexOf(needle) !== -1);
      };
      const coderMsgsView = filterMsgs(coderMsgs, coderFind);
      const observerMsgsView = filterMsgs(observerMsgs, observerFind);
      const threadTasks = (activeThread && Array.isArray(activeThread.tasks)) ? activeThread.tasks : [];
      const sortedTasks = [...threadTasks].sort((a, b) => {
        const sa = String(a && a.status ? a.status : "new");
        const sb = String(b && b.status ? b.status : "new");
        if (sa !== sb) {
          if (sa === "done") return 1;
          if (sb === "done") return -1;
          if (sa === "sent") return 1;
          if (sb === "sent") return -1;
        }
        const pa = typeof a.priority === "number" ? a.priority : 50;
        const pb = typeof b.priority === "number" ? b.priority : 50;
        if (pa !== pb) return pb - pa;
        return (a && a.createdAt ? a.createdAt : 0) - (b && b.createdAt ? b.createdAt : 0);
      });
      const normTitle = (s) => String(s || "").trim().toLowerCase().replace(/\s+/g, " ");
      const approvedProposalTitles = (() => {
        const out = new Set();
        for (const m of coderMsgs || []) {
          if (!m || m.role !== "user") continue;
          const t = String(m.content || "");
          if (!t.startsWith("[Observer proposal approved]")) continue;
          const mm = t.match(/^Title:\s*(.+)$/im);
          const title = mm ? mm[1] : "";
          const k = normTitle(title);
          if (k) out.add(k);
        }
        return out;
      })();
      let lastObserverAsst = null;
      for (let i = observerMsgs.length - 1; i >= 0; i--) {
        if (observerMsgs[i].role === "assistant") {
          lastObserverAsst = observerMsgs[i];
        break;
      }
    }
    const observerProposals = parseProposals(lastObserverAsst ? lastObserverAsst.content : "");
    const criticalPath = parseCriticalPath(lastObserverAsst ? lastObserverAsst.content : "");
    const healthScore = parseHealthScore(lastObserverAsst ? lastObserverAsst.content : "");

    // Detect current development phase from the latest Observer message that contains one.
    const observerPhase = (() => {
      for (let i = observerMsgs.length - 1; i >= 0; i--) {
        const m = observerMsgs[i];
        if (m.role === "assistant" && !m.streaming) {
          const p = parsePhase(String(m.content || ""));
          if (p) return p;
        }
      }
      return null;
    })();

    // Sort proposals: phase-match first, then by score descending.
    const sortedProposals = [...observerProposals].sort((a, b) => {
      const aOk = !observerPhase || a.phase === "any" || a.phase === observerPhase;
      const bOk = !observerPhase || b.phase === "any" || b.phase === observerPhase;
      if (aOk !== bOk) return aOk ? -1 : 1;
      return (b.score || 50) - (a.score || 50);
    });

    return e(
      "div",
      { className: "app" },
      e(
        "div",
        { className: "topbar" },
        e(
          "div",
          { className: "topbar-inner" },
          e(
            "div",
            { className: "brand" },
            e("h1", null, "OBSTRAL"),
            e(
              "span",
              { className: "pill", title: status && status.workspace_root ? String(status.workspace_root) : "" },
              status && status.version ? `v${status.version}` : "local"
            ),
            serverOutdated
              ? e(
                  "span",
                  { className: "pill pill-warn", title: "Restart the lite server to pick up latest backend features." },
                  tr(lang, "serverOutdated")
                )
              : null,
            e(
              "span",
              { className: "pill", title: tr(lang, "keys") },
              e(KeyDot, { ok: keyCodestralOk }),
              " C  ",
              e(KeyDot, { ok: keyMistralOk }),
              " M  ",
              e(KeyDot, { ok: keyOpenAIOk }),
              " O  ",
              e(KeyDot, { ok: keyGeminiOk }),
              " G  ",
              e(KeyDot, { ok: keyAnthropicOk }),
              " A"
            )
          ),
          e(
            "div",
            { className: "top-actions" },
            e(
              "div",
              { className: "seg" },
              e("button", { className: "seg-btn " + (lang === "ja" ? "active" : ""), onClick: () => setLang("ja") }, "JA"),
              e("button", { className: "seg-btn " + (lang === "en" ? "active" : ""), onClick: () => setLang("en") }, "EN"),
              e("button", { className: "seg-btn " + (lang === "fr" ? "active" : ""), onClick: () => setLang("fr") }, "FR")
            ),
            e("button", { className: "btn", onClick: refreshStatus, type: "button" }, tr(lang, "refresh")),
            e("button", {
              className: "btn btn-icon",
              title: tr(lang, "shortcuts") + " (?)",
              onClick: () => setShowShortcuts((v) => !v),
              type: "button",
            }, "⌨")
          )
        )
      ),
      e(
        "datalist",
        { id: "models-list" },
        (models || []).map((m) => e("option", { key: m, value: m }))
      ),
      e(
        "div",
        { className: "main" },
        e(
          "div",
          { style: { display: "flex", flexDirection: "column", gap: "14px" } },
          e(
            "div",
            { className: "panel" },
            e(
              "div",
              { className: "panel-header" },
              e("h2", null, tr(lang, "threads")),
              e(
                "div",
                { style: { display: "flex", gap: "8px" } },
                e("button", { className: "btn btn-primary", onClick: createThread }, tr(lang, "newThread")),
                e("button", { className: "btn", onClick: exportMd }, tr(lang, "exportMd"))
              )
            ),
            e(
              "div",
              { className: "panel-body" },
              e(
                "div",
                { style: { display: "flex", flexDirection: "column", gap: "10px", maxHeight: "260px", overflow: "auto" } },
                threadState.threads.map((t) => {
                  const active = t.id === threadState.activeId;
                  const activeStyle = active
                    ? { borderColor: "rgba(96,165,250,0.60)", background: "rgba(96,165,250,0.10)" }
                    : null;
                  const msgs = (t.messages || []);
                  const lastMsg = msgs.length ? msgs[msgs.length - 1] : null;
                  const workdirLabel = t.workdir ? (" · wd:" + String(t.workdir)) : "";
                  const snippet = (() => {
                    if (!lastMsg || !lastMsg.content) return "";
                    const raw = String(lastMsg.content || "").replace(/\r\n/g, "\n").trim();
                    if (!raw) return "";
                    const firstLine = raw.split("\n").find((x) => String(x || "").trim()) || raw;
                    const oneLine = String(firstLine).replace(/\s+/g, " ").trim();
                    const max = 120;
                    return oneLine.length > max ? (oneLine.slice(0, max) + "...") : oneLine;
                  })();
                  const snippetPrefix = (() => {
                    if (!lastMsg) return "";
                    const p = lastMsg.pane === "observer" ? "O" : lastMsg.pane === "chat" ? "H" : "C";
                    const r = lastMsg.role === "user" ? "U" : "A";
                    return p + r + ": ";
                  })();
                  const isEditing = editingThreadId === t.id;
                  const isConfirmDel = confirmDeleteId === t.id;
                  return e(
                    "div",
                    { key: t.id, style: { display: "flex", gap: "8px", alignItems: "stretch" } },
                    isEditing
                      ? e(
                          "div",
                          { style: { flex: 1, display: "flex", gap: 6 } },
                          e("input", {
                            className: "field",
                            style: { flex: 1, padding: "6px 8px", fontSize: 13 },
                            value: editingTitle,
                            autoFocus: true,
                            onChange: (ev) => setEditingTitle(ev.target.value),
                            onKeyDown: (ev) => {
                              if (ev.key === "Enter") commitRename(t.id);
                              if (ev.key === "Escape") setEditingThreadId(null);
                            },
                          }),
                          e("button", { className: "btn btn-accent", style: { padding: "6px 10px" }, onClick: () => commitRename(t.id) }, "✓"),
                          e("button", { className: "btn", style: { padding: "6px 10px" }, onClick: () => setEditingThreadId(null) }, "✕")
                        )
                      : isConfirmDel
                      ? e(
                          "div",
                          { style: { flex: 1, display: "flex", gap: 6, alignItems: "center" } },
                          e("span", { style: { flex: 1, fontSize: 12, color: "var(--danger,#f87171)" } }, tr(lang, "delQ")),
                          e("button", { className: "btn btn-warn", style: { padding: "6px 10px" }, onClick: () => confirmDelete(t.id) }, tr(lang, "yes")),
                          e("button", { className: "btn", style: { padding: "6px 10px" }, onClick: () => setConfirmDeleteId(null) }, tr(lang, "no"))
                        )
                      : e(
                          "button",
                          {
                            className: "preset",
                            style: { flex: 1, ...(activeStyle || {}) },
                            onClick: () => setThreadState((s) => ({ ...s, activeId: t.id })),
                          },
                          e("div", { className: "preset-title" }, t.title),
                          e(
                            "div",
                            { className: "preset-sub" },
                            `${new Date(t.updatedAt).toLocaleString()} · ${msgs.length} msgs${workdirLabel}`
                          ),
                          snippet
                            ? e("div", { className: "preset-snippet" }, snippetPrefix + snippet)
                            : null
                        ),
                    !isEditing && !isConfirmDel && e("button", { className: "btn", style: { padding: "8px 10px" }, onClick: () => renameThread(t.id) }, "✎"),
                    !isEditing && !isConfirmDel && e(
                      "button",
                      { className: "btn btn-warn", style: { padding: "8px 10px" }, onClick: () => deleteThread(t.id) },
                      "×"
                    )
                  );
                })
              )
            )
          ),
          e(
            "div",
            { className: "panel" },
            e("div", { className: "panel-header" }, e("h2", null, tr(lang, "settings"))),
            e(
              "div",
              { className: "panel-body" },
              e("div", { className: "section-title" }, tr(lang, "presets")),
              e(
                "div",
                { className: "preset-row" },
                e(
                  "button",
                  { className: "preset preset-vibe", onClick: () => applyPreset("vibe") },
                  e("div", { className: "preset-title" }, "VIBE"),
                  e("div", { className: "preset-sub" }, "Mistral · codestral-latest")
                ),
                e(
                  "button",
                  { className: "preset", onClick: () => applyPreset("codestral") },
                  e("div", { className: "preset-title" }, "Codestral"),
                  e("div", { className: "preset-sub" }, "codestral.mistral.ai · /v1")
                ),
                e(
                  "button",
                  { className: "preset", onClick: () => applyPreset("openai") },
                  e("div", { className: "preset-title" }, "OpenAI"),
                  e("div", { className: "preset-sub" }, "OpenAI-compatible · /v1")
                ),
                e(
                  "button",
                  { className: "preset", onClick: () => applyPreset("anthropic") },
                  e("div", { className: "preset-title" }, "Anthropic"),
                  e("div", { className: "preset-sub" }, "/v1/messages")
                )
              ),
              e("div", { className: "hr" }),
              e(
                "div",
                { className: "grid2" },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "provider")),
                  e(
                    "select",
                    {
                      className: "select",
                      value: config.provider,
                      onChange: (ev) => setProviderSafe(ev.target.value),
                    },
                    PROVIDERS.map((p) =>
                      e("option", { key: p.k, value: p.k }, (p.l && p.l[lang]) || p.k)
                    )
                  )
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "chatModel")),
                  e("input", {
                    className: "input",
                    value: config.chatModel || "",
                    list: models && models.length ? "models-list" : undefined,
                    onChange: (ev) => setConfig({ ...config, chatModel: ev.target.value }),
                  })
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "codeModel")),
                e("input", {
                  className: "input",
                  value: config.codeModel || "",
                  list: models && models.length ? "models-list" : undefined,
                  onChange: (ev) => setConfig({ ...config, codeModel: ev.target.value }),
                })
              ),
              e(
                "div",
                { style: { display: "flex", gap: 10, alignItems: "center", flexWrap: "wrap" } },
                e(
                  "div",
                  { className: "seg" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (modelsTarget === "chat" ? "active" : ""),
                      onClick: () => setModelsTarget("chat"),
                      type: "button",
                    },
                    "Chat"
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (modelsTarget === "code" ? "active" : ""),
                      onClick: () => setModelsTarget("code"),
                      type: "button",
                    },
                    "Code"
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (modelsTarget === "observer" ? "active" : ""),
                      onClick: () => setModelsTarget("observer"),
                      type: "button",
                    },
                    "Obs"
                  )
                ),
                e(
                  "button",
                  { className: "btn", onClick: fetchModels, disabled: modelsLoading },
                  modelsLoading ? "Loading..." : tr(lang, "fetchModels")
                ),
                models && models.length
                  ? e("span", { className: "pill" }, `${models.length} models`)
                  : null
              ),
              modelsErr ? e("div", { className: "hint", style: { color: "var(--warn)", marginTop: "8px" } }, modelsErr) : null,
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "baseUrl")),
                e("input", {
                  className: "input",
                  value: config.baseUrl,
                  onChange: (ev) => setConfig({ ...config, baseUrl: ev.target.value }),
                })
              ),
              e(
                "div",
                { className: "grid2", style: { marginTop: "10px" } },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "codeProvider")),
                  e(
                    "select",
                    {
                      className: "select",
                      value: String(config.codeProvider || ""),
                      onChange: (ev) => setConfig({ ...config, codeProvider: ev.target.value }),
                    },
                    e("option", { value: "" }, tr(lang, "sameAsChat")),
                    PROVIDERS.map((p) => e("option", { key: p.k, value: p.k }, (p.l && p.l[lang]) || p.k))
                  )
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "codeBaseUrl")),
                  e("input", {
                    className: "input",
                    value: String(config.codeBaseUrl || ""),
                    onChange: (ev) => setConfig({ ...config, codeBaseUrl: ev.target.value }),
                    placeholder: tr(lang, "sameAsChat"),
                  })
                )
              ),
              e(
                "div",
                { className: "grid2", style: { marginTop: "10px" } },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "mode")),
                  e(
                    "select",
                    {
                      className: "select",
                      value: config.mode,
                      onChange: (ev) => setConfig({ ...config, mode: ev.target.value }),
                    },
                    CODER_MODES.map((m) => e("option", { key: m.k, value: m.k }, (m.l && m.l[lang]) || m.k))
                  )
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "persona")),
                  e(
                    "select",
                    {
                      className: "select",
                      value: config.persona,
                      onChange: (ev) => setConfig({ ...config, persona: ev.target.value }),
                    },
                    PERSONAS.map((p) => e("option", { key: p, value: p }, p))
                  )
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "observerIntensity")),
                e(
                  "div",
                  { className: "seg seg-4" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (String(config.observerIntensity || "critical") === "polite" ? "active" : ""),
                      onClick: () => setConfig({ ...config, observerIntensity: "polite" }),
                      type: "button",
                    },
                    tr(lang, "polite")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (String(config.observerIntensity || "critical") === "critical" ? "active" : ""),
                      onClick: () => setConfig({ ...config, observerIntensity: "critical" }),
                      type: "button",
                    },
                    tr(lang, "critical")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (String(config.observerIntensity || "critical") === "brutal" ? "active" : ""),
                      onClick: () => setConfig({ ...config, observerIntensity: "brutal" }),
                      type: "button",
                    },
                    tr(lang, "brutal")
                  )
                )
              ),
              e(
                "div",
                { className: "grid2", style: { marginTop: "10px" } },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "observer") + " · " + tr(lang, "mode")),
                  e(
                    "select",
                    {
                      className: "select",
                      value: config.observerMode,
                      onChange: (ev) => setConfig({ ...config, observerMode: ev.target.value }),
                    },
                    OBSERVER_MODES.map((m) => e("option", { key: m.k, value: m.k }, (m.l && m.l[lang]) || m.k))
                  )
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "observer") + " · " + tr(lang, "persona")),
                  e(
                    "select",
                    {
                      className: "select",
                      value: config.observerPersona,
                      onChange: (ev) => setConfig({ ...config, observerPersona: ev.target.value }),
                    },
                    PERSONAS.map((p) => e("option", { key: p, value: p }, p))
                  )
                )
              ),
              e(
                "div",
                { className: "grid2", style: { marginTop: "10px" } },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "observerProvider")),
                  e(
                    "select",
                    {
                      className: "select",
                      value: String(config.observerProvider || ""),
                      onChange: (ev) => setConfig({ ...config, observerProvider: ev.target.value }),
                    },
                    e("option", { value: "" }, tr(lang, "sameAsChat")),
                    PROVIDERS.map((p) => e("option", { key: p.k, value: p.k }, (p.l && p.l[lang]) || p.k))
                  )
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "observerModel")),
                  e("input", {
                    className: "input",
                    value: String(config.observerModel || ""),
                    list: models && models.length ? "models-list" : undefined,
                    onChange: (ev) => setConfig({ ...config, observerModel: ev.target.value }),
                    placeholder: tr(lang, "sameAsChat"),
                  })
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "observerBaseUrl")),
                e("input", {
                  className: "input",
                  value: String(config.observerBaseUrl || ""),
                  onChange: (ev) => setConfig({ ...config, observerBaseUrl: ev.target.value }),
                  placeholder: tr(lang, "sameAsChat"),
                })
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "cot")),
                e(
                  "div",
                  { className: "seg" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (String(config.cot || "brief") === "brief" ? "active" : ""),
                      onClick: () => setConfig({ ...config, cot: "brief" }),
                      type: "button",
                    },
                    tr(lang, "brief")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (String(config.cot || "brief") === "structured" ? "active" : ""),
                      onClick: () => setConfig({ ...config, cot: "structured" }),
                      type: "button",
                    },
                    tr(lang, "structured")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (String(config.cot || "brief") === "deep" ? "active" : ""),
                      onClick: () => setConfig({ ...config, cot: "deep" }),
                      type: "button",
                    },
                    tr(lang, "deep")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (String(config.cot || "brief") === "off" ? "active" : ""),
                      onClick: () => setConfig({ ...config, cot: "off" }),
                      type: "button",
                    },
                    tr(lang, "off")
                  )
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "editApproval")),
                e(
                  "div",
                  { className: "seg" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (config.requireEditApproval ? "active" : ""),
                      onClick: () => setConfig({ ...config, requireEditApproval: true }),
                      type: "button",
                    },
                    tr(lang, "on")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (!config.requireEditApproval ? "active" : ""),
                      onClick: () => setConfig({ ...config, requireEditApproval: false }),
                      type: "button",
                    },
                    tr(lang, "off")
                  )
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "commandApproval")),
                e(
                  "div",
                  { className: "seg" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (config.requireCommandApproval ? "active" : ""),
                      onClick: () => setConfig({ ...config, requireCommandApproval: true }),
                      type: "button",
                    },
                    tr(lang, "on")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (!config.requireCommandApproval ? "active" : ""),
                      onClick: () => setConfig({ ...config, requireCommandApproval: false }),
                      type: "button",
                    },
                    tr(lang, "off")
                  )
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "autoObserve")),
                e(
                  "div",
                  { className: "seg" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (config.autoObserve ? "active" : ""),
                      onClick: () => setConfig({ ...config, autoObserve: true }),
                      type: "button",
                    },
                    tr(lang, "on")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (!config.autoObserve ? "active" : ""),
                      onClick: () => setConfig({ ...config, autoObserve: false }),
                      type: "button",
                    },
                    tr(lang, "off")
                  )
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "forceAgent")),
                e(
                  "div",
                  { className: "seg" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (config.forceAgent ? "active" : ""),
                      onClick: () => setConfig({ ...config, forceAgent: true }),
                      type: "button",
                    },
                    tr(lang, "on")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (!config.forceAgent ? "active" : ""),
                      onClick: () => setConfig({ ...config, forceAgent: false }),
                      type: "button",
                    },
                    tr(lang, "off")
                  )
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "toolRoot")),
                e("input", {
                  className: "input",
                  value: String(config.toolRoot || ""),
                  onChange: (ev) => setConfig({ ...config, toolRoot: ev.target.value }),
                  placeholder: "(optional) subdir (e.g. myrepo)",
                })
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "workdir")),
                e("input", {
                  className: "input",
                  value: String((activeThread && activeThread.workdir) || ""),
                  onChange: (ev) => {
                    const v = String(ev.target.value || "");
                    setThreadState((s) => ({
                      ...s,
                      threads: s.threads.map((t) => (t.id === (activeThread && activeThread.id) ? { ...t, updatedAt: Date.now(), workdir: v } : t)),
                    }));
                  },
                  placeholder: "(optional) subdir inside tool_root (e.g. myrepo)",
                })
              ),
              (String(config.provider || "").trim() === "mistral-cli" ||
                String(config.codeProvider || "").trim() === "mistral-cli" ||
                String(config.observerProvider || "").trim() === "mistral-cli")
                ? e(
                    "div",
                    { className: "grid2", style: { marginTop: "10px" } },
                    e(
                      "div",
                      { className: "field" },
                      e("label", null, tr(lang, "vibeAgent")),
                      e("input", {
                        className: "input",
                        value: String(config.mistralCliAgent || ""),
                        onChange: (ev) => setConfig({ ...config, mistralCliAgent: ev.target.value }),
                        placeholder: "accept-edits / plan / ...",
                      })
                    ),
                    e(
                      "div",
                      { className: "field" },
                      e("label", null, tr(lang, "vibeMaxTurns")),
                      e("input", {
                        className: "input",
                        value: String(config.mistralCliMaxTurns || ""),
                        onChange: (ev) => setConfig({ ...config, mistralCliMaxTurns: ev.target.value }),
                        placeholder: "8",
                        inputMode: "numeric",
                      })
                    )
                  )
                : null,
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "includeCoderContext")),
                e(
                  "div",
                  { className: "seg" },
                  e(
                    "button",
                    {
                      className: "seg-btn " + (config.includeCoderContext ? "active" : ""),
                      onClick: () => setConfig({ ...config, includeCoderContext: true }),
                      type: "button",
                    },
                    tr(lang, "on")
                  ),
                  e(
                    "button",
                    {
                      className: "seg-btn " + (!config.includeCoderContext ? "active" : ""),
                      onClick: () => setConfig({ ...config, includeCoderContext: false }),
                      type: "button",
                    },
                    tr(lang, "off")
                  )
                )
              ),
              e(
                "div",
                { className: "grid2", style: { marginTop: "10px" } },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "temperature")),
                  e("input", {
                    className: "input",
                    value: config.temperature,
                    onChange: (ev) => setConfig({ ...config, temperature: ev.target.value }),
                    inputMode: "decimal",
                  })
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "maxTokens")),
                  e("input", {
                    className: "input",
                    value: config.maxTokens,
                    onChange: (ev) => setConfig({ ...config, maxTokens: ev.target.value }),
                    inputMode: "numeric",
                  })
                )
              ),
              e(
                "div",
                { className: "grid2", style: { marginTop: "10px" } },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "timeoutSeconds")),
                  e("input", {
                    className: "input",
                    value: config.timeoutSeconds,
                    onChange: (ev) => setConfig({ ...config, timeoutSeconds: ev.target.value }),
                    inputMode: "numeric",
                  })
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "stream")),
                  e(
                    "div",
                    { className: "seg" },
                    e(
                      "button",
                      {
                        className: "seg-btn " + (config.stream ? "active" : ""),
                        onClick: () => setConfig({ ...config, stream: true }),
                      },
                      tr(lang, "on")
                    ),
                    e(
                      "button",
                      {
                        className: "seg-btn " + (!config.stream ? "active" : ""),
                        onClick: () => setConfig({ ...config, stream: false }),
                      },
                      tr(lang, "off")
                    )
                  )
                )
              ),
              e(
                "div",
                { className: "grid2", style: { marginTop: "10px" } },
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "apiKeyChat")),
                  e("input", {
                    className: "input",
                    type: "password",
                    value: chatApiKey,
                    onChange: (ev) => setChatApiKey(ev.target.value),
                    placeholder: "env",
                  })
                ),
                e(
                  "div",
                  { className: "field" },
                  e("label", null, tr(lang, "apiKeyCode")),
                  e("input", {
                    className: "input",
                    type: "password",
                    value: codeApiKey,
                    onChange: (ev) => setCodeApiKey(ev.target.value),
                    placeholder: "env",
                  })
                )
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "apiKeyObserver")),
                e("input", {
                  className: "input",
                  type: "password",
                  value: observerApiKey,
                  onChange: (ev) => setObserverApiKey(ev.target.value),
                  placeholder: "env",
                })
              ),
              e(
                "div",
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "pendingEdits")),
                pendingEdits && pendingEdits.length
                  ? e(
                      "div",
                      { style: { display: "flex", flexDirection: "column", gap: 8, maxHeight: 160, overflow: "auto" } },
                      pendingEdits.map((it) =>
                        e(
                          "div",
                          {
                            key: String(it.id || Math.random()),
                            style: {
                              border: "1px solid rgba(255,255,255,0.15)",
                              borderRadius: 8,
                              padding: "8px 10px",
                              background: "rgba(255,255,255,0.03)",
                            },
                          },
                          e(
                            "div",
                            { style: { display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" } },
                            e("code", null, String(it.action || "")),
                            e("span", { className: "pill" }, String(it.status || "")),
                            e("span", { style: { color: "var(--faint)", fontSize: 12 } }, String(it.path || "")),
                            String(it.status || "") === "pending"
                              ? e(
                                  "div",
                                  { style: { marginLeft: "auto", display: "flex", gap: 6 } },
                                  e(
                                    "button",
                                    {
                                      className: "btn btn-primary",
                                      type: "button",
                                      disabled: pendingBusy,
                                      onClick: () => resolvePendingEdit(it.id, true),
                                    },
                                    tr(lang, "approve")
                                  ),
                                  e(
                                    "button",
                                    {
                                      className: "btn btn-warn",
                                      type: "button",
                                      disabled: pendingBusy,
                                      onClick: () => resolvePendingEdit(it.id, false),
                                    },
                                    tr(lang, "reject")
                                  )
                                )
                              : null
                          ),
                          it.preview
                            ? e(
                                "pre",
                                { style: { marginTop: 6, maxHeight: 100, overflow: "auto" } },
                                String(it.diff || it.preview)
                              )
                            : null
                        )
                      )
                    )
                  : e("div", { className: "hint" }, "none")
              ),
              e(
                "div",
                {
                  className: "diffbox",
                  onDragOver: (ev) => { ev.preventDefault(); ev.currentTarget.style.borderColor = "rgba(45,212,191,0.7)"; ev.currentTarget.style.background = "rgba(45,212,191,0.07)"; },
                  onDragLeave: (ev) => { ev.currentTarget.style.borderColor = ""; ev.currentTarget.style.background = ""; },
                  onDrop: async (ev) => {
                    ev.preventDefault();
                    ev.currentTarget.style.borderColor = "";
                    ev.currentTarget.style.background = "";
                    const f = ev.dataTransfer && ev.dataTransfer.files && ev.dataTransfer.files[0];
                    if (!f) return;
                    const txt = await readFileText(f);
                    setDiff(txt);
                    setConfig((c) => ({ ...c, mode: "diff批評" }));
                  },
                },
                e("div", { className: "section-title" }, tr(lang, "diff") + "  (⎘ drag & drop)"),
                e(
                  "div",
                  { className: "diff-actions" },
                  e("input", {
                    className: "file",
                    type: "file",
                    accept: ".diff,.patch,.txt",
                    onChange: async (ev) => {
                      const f = ev.target.files && ev.target.files[0];
                      if (!f) return;
                      const txt = await readFileText(f);
                      setDiff(txt);
                      ev.target.value = "";
                    },
                  }),
                  e("button", { className: "btn", onClick: () => setDiff("") }, tr(lang, "clear"))
                ),
                e("textarea", {
                  className: "textarea",
                  value: diff,
                  onChange: (ev) => setDiff(ev.target.value),
                  placeholder: tr(lang, "diffHint"),
                })
              )
            )
          )
        ),
        e(
          "div",
          { className: "arena", ref: arenaRef, style: { display: "flex", gap: 0 } },
          // Coder pane
          e(
            "div",
            { className: "panel chat", style: { flex: splitPct, minWidth: 0 } },
            e(
              "div",
              { className: "panel-header" },
              e("h2", null, tr(lang, "coder") + ": " + (activeThread ? activeThread.title : "")),
              e(
                "div",
                { style: { display: "flex", gap: 8, alignItems: "center" } },
                e("span", {
                  style: {
                    width: 8, height: 8, borderRadius: "50%", display: "inline-block",
                    background: PROVIDER_COLORS[coderResolvedProvider(config.mode)] || "rgba(255,255,255,0.4)",
                    boxShadow: "0 0 8px " + (PROVIDER_COLORS[coderResolvedProvider(config.mode)] || "transparent") + "88",
                    transition: "background 400ms ease, box-shadow 400ms ease",
                  },
                }),
                e(
                  "span",
                  { className: "pill", title: coderActiveModel() },
                  providerLabel(coderResolvedProvider(config.mode)) + " · " + modeLabel(config.mode) + " · " + shortModel(coderActiveModel())
                )
              )
            ),
            e(
              "div",
              { className: "chat-find" },
              e("input", {
                className: "input",
                value: coderFind,
                placeholder: tr(lang, "findInThread"),
                onChange: (ev) => setCoderFind(ev.target.value),
                onKeyDown: (ev) => { if (ev.key === "Escape") setCoderFind(""); },
                style: { flex: 1, padding: "8px 10px", fontSize: 12, fontFamily: "var(--mono)" },
              }),
              coderFind
                ? e("button", { className: "btn btn-icon", type: "button", title: tr(lang, "clear"), onClick: () => setCoderFind("") }, "✕")
                : null,
              coderFind
                ? e("span", { className: "msg-ts", style: { marginLeft: 6 } }, `${coderMsgsView.length}/${coderMsgs.length}`)
                : null
            ),
            e("div", { className: "chat-body", ref: coderBodyRef },
              coderMsgs.length === 0
                ? e("div", { className: "pane-empty" },
                    e("div", { className: "pane-empty-icon" }, "⚡"),
                    e("p", { className: "pane-empty-hint" }, tr(lang, "placeholder"))
                  )
                : coderMsgsView.length === 0
                  ? e("div", { className: "pane-empty" },
                      e("div", { className: "pane-empty-icon" }, "🔎"),
                      e("p", { className: "pane-empty-hint" }, tr(lang, "noMatches"))
                    )
                  : coderMsgsView.map(renderMessage)
            ),
            e(
              "div",
              { className: "composer" },
              e("textarea", {
                className: "textarea",
                value: coderInput,
                rows: Math.max(2, Math.min(8, (coderInput.match(/\n/g) || []).length + 1)),
                style: { resize: "none" },
                placeholder: tr(lang, "placeholder"),
                onChange: (ev) => setCoderInput(ev.target.value),
                onKeyDown: (ev) => {
                  if ((ev.metaKey || ev.ctrlKey) && ev.key === "Enter") {
                    ev.preventDefault();
                    sendCoder();
                    return;
                  }
                  if (ev.key === "Enter" && !ev.shiftKey) {
                    ev.preventDefault();
                    sendCoder();
                  }
                },
              }),
              e(
                "div",
                { style: { display: "flex", gap: 8, alignItems: "center" } },
                e(
                  "button",
                  { className: "btn", type: "button", onClick: insertCliTemplate, disabled: sendingCoder },
                  tr(lang, "insertCliTemplate")
                ),
                sendingCoder
                  ? e("button", { className: "btn btn-warn", onClick: stopCoder }, tr(lang, "stop"))
                  : e("button", { className: "btn btn-primary", onClick: () => sendCoder() }, tr(lang, "send"))
              )
            ),
            e(
              "div",
              { className: "statusline" },
              e("span", {
                className: "dot" + (sendingCoder ? " streaming" : ""),
                style: {
                  background: sendingCoder
                    ? (PROVIDER_COLORS[coderResolvedProvider(config.mode)] || "rgba(45,212,191,0.85)")
                    : "rgba(45,212,191,0.85)",
                  transition: "background 400ms ease",
                },
              }),
              e("span", null, sendingCoder ? (config.stream ? tr(lang, "streaming") : tr(lang, "sending")) : tr(lang, "ready")),
              coderMsgs.length > 0 && e(
                "span",
                {
                  style: { marginLeft: "auto", color: "var(--faint)", fontSize: 11, fontFamily: "var(--mono)" },
                },
                "~" + coderMsgs.reduce((s, m) => s + estimateTokens(m.content), 0) + " tokens · " + coderMsgs.length + " msgs"
              )
            )
          ),

          // Drag handle
          e("div", { className: "drag-handle", onMouseDown: onSplitDragStart }),

          // Observer pane
          e(
            "div",
            { className: "panel chat", style: { flex: 100 - splitPct, minWidth: 0 } },
            e(
              "div",
              { className: "panel-header" },
              e("h2", null, tr(lang, "observer")),
              e(
                "div",
                { style: { display: "flex", gap: 8, alignItems: "center" } },
                e("span", {
                  style: {
                    width: 8, height: 8, borderRadius: "50%", display: "inline-block",
                    background: PROVIDER_COLORS[observerResolvedProvider()] || "rgba(251, 191, 36, 0.85)",
                    boxShadow: "0 0 8px " + (PROVIDER_COLORS[observerResolvedProvider()] || "rgba(251, 191, 36, 0.45)") + "88",
                    transition: "background 400ms ease, box-shadow 400ms ease",
                  },
                }),
                e(
                  "span",
                  { className: "pill", title: observerActiveModel() },
                  providerLabel(observerResolvedProvider()) + " · " + modeLabel(config.observerMode) + " · " + shortModel(observerActiveModel())
                ),
                loopInfo && loopInfo.depth > 0
                  ? e(
                      "span",
                      {
                        className: "pill pill-warn",
                        title: `${tr(lang, "loopDetected")} (sim=${Math.round((loopInfo.score || 0) * 100)}%)`,
                      },
                      tr(lang, "loopDetected") + " ×" + String(loopInfo.depth)
                    )
                  : null,
                observerPhase && e("span", { className: "phase-indicator phase-" + observerPhase }, observerPhase),
                config.autoObserve && e("span", { className: "pill auto-badge" }, "AUTO")
              )
            ),
            e("div", { className: "obs-subtab-bar" },
              e("button", {
                className: "obs-subtab" + (observerSubTab === "analysis" ? " active" : ""),
                onClick: () => setObserverSubTab("analysis"),
              }, tr(lang, "observer")),
              e("button", {
                className: "obs-subtab" + (observerSubTab === "chat" ? " active" : ""),
                onClick: () => setObserverSubTab("chat"),
              }, tr(lang, "chat"))
            ),
            observerSubTab === "chat"
              ? e(React.Fragment, { key: "chat" },
                  e("div", { className: "obs-scroll-zone chat-scroll-zone" },
                    e("div", { className: "chat-body", ref: chatBodyRef },
                      paneMessages("chat").length === 0
                        ? e("div", { className: "pane-empty chat-empty" },
                            e("div", { className: "pane-empty-icon" }, "💬"),
                            e("div", { className: "chat-empty-title" },
                              lang === "en" ? "Free chat"
                              : lang === "fr" ? "Discussion libre"
                              : "会話モード"
                            ),
                            e("p", { className: "pane-empty-hint" },
                              lang === "en" ? "Ask anything — code questions, design decisions, or just think out loud"
                              : lang === "fr" ? "Posez n'importe quelle question — code, design, ou simple réflexion"
                              : "なんでも聞いてください — 実装の疑問、設計の相談、ゴム鴨モード"
                            )
                          )
                        : paneMessages("chat").map(renderMessage)
                    )
                  ),
                  e("div", { className: "chat-persona-bar" },
                    CHAT_PERSONAS.map((p) =>
                      e("button", {
                        key: p.key,
                        className: "persona-chip" + (config.chatPersona === p.key ? " active" : ""),
                        onClick: () => setConfig({ ...config, chatPersona: p.key }),
                        title: lang === "en" ? p.en : lang === "fr" ? p.fr : p.ja,
                      }, p.icon + "\u00a0" + (lang === "en" ? p.en : lang === "fr" ? p.fr : p.ja))
                    )
                  ),
                  e("div", { className: "composer chat-composer" },
                    e("textarea", {
                      className: "textarea",
                      value: chatInput,
                      placeholder: lang === "en" ? "Chat…" : lang === "fr" ? "Discuter…" : "話しかける…",
                      rows: Math.max(2, Math.min(8, (chatInput.match(/\n/g) || []).length + 1)),
                      style: { resize: "none" },
                      onChange: (ev) => setChatInput(ev.target.value),
                      onKeyDown: (ev) => {
                        if ((ev.metaKey || ev.ctrlKey) && ev.key === "Enter") { ev.preventDefault(); sendChat(); return; }
                        if (ev.key === "Enter" && !ev.shiftKey) { ev.preventDefault(); sendChat(); }
                      },
                    }),
                    sendingChat
                      ? e("button", { className: "btn btn-warn", onClick: stopChat }, tr(lang, "stop"))
                      : e("button", { className: "btn btn-primary", onClick: () => sendChat() }, tr(lang, "send"))
                  ),
                  e("div", { className: "statusline" },
                    e("span", { className: "dot" + (sendingChat ? " streaming" : "") }),
                    e("span", null, sendingChat ? (config.stream ? tr(lang, "streaming") : tr(lang, "sending")) : tr(lang, "ready")),
                    paneMessages("chat").length > 0 && e("span", {
                      style: { marginLeft: "auto", color: "var(--faint)", fontSize: 11, fontFamily: "var(--mono)" },
                    }, paneMessages("chat").length + " msgs")
                  )
                )
              : e(React.Fragment, { key: "analysis" },
                  e("div", { className: "obs-scroll-zone" },
                  e(
                    "div",
                    { className: "chat-find" },
                    e("input", {
                      className: "input",
                      value: observerFind,
                      placeholder: tr(lang, "findInThread"),
                      onChange: (ev) => setObserverFind(ev.target.value),
                      onKeyDown: (ev) => { if (ev.key === "Escape") setObserverFind(""); },
                      style: { flex: 1, padding: "8px 10px", fontSize: 12, fontFamily: "var(--mono)" },
                    }),
                    observerFind
                      ? e("button", { className: "btn btn-icon", type: "button", title: tr(lang, "clear"), onClick: () => setObserverFind("") }, "✕")
                      : null,
                    observerFind
                      ? e("span", { className: "msg-ts", style: { marginLeft: 6 } }, `${observerMsgsView.length}/${observerMsgs.length}`)
                      : null
                  ),
                  e("div", { className: "chat-body", ref: observerBodyRef },
                    observerMsgs.length === 0
                      ? e("div", { className: "pane-empty pane-empty-obs" },
                          e("div", { className: "pane-empty-icon" }, "👁"),
                          e("p", { className: "pane-empty-hint" }, tr(lang, "observerHint")),
                          !config.autoObserve && coderMsgs.length > 0 && e("button", {
                            className: "btn btn-accent obs-quick-trigger",
                            onClick: () => sendObserver(lang === "en"
                              ? "Please review the Coder's latest output."
                              : lang === "fr"
                                ? "Veuillez examiner la dernière sortie du Coder."
                                : "Coderの最新の出力をレビューしてください。"),
                          }, lang === "en" ? "▶ Observe now" : lang === "fr" ? "▶ Observer" : "▶ 今すぐ観察")
                        )
                      : observerMsgsView.length === 0
                        ? e("div", { className: "pane-empty" },
                            e("div", { className: "pane-empty-icon" }, "🔎"),
                            e("p", { className: "pane-empty-hint" }, tr(lang, "noMatches"))
                          )
                        : observerMsgsView.map(renderMessage)
                  ),
                  criticalPath
                    ? e("div", { className: "critical-path-banner" },
                        e("span", { className: "critical-path-icon" }, "⚠"),
                        e("span", { className: "critical-path-text" }, criticalPath)
                      )
                    : null,
                  sortedTasks && sortedTasks.length
                    ? e(
                        "div",
                        { className: "taskbox" },
                        e("div", { className: "section-title", style: { margin: 0, display: "flex", alignItems: "center", gap: 10 } },
                          tr(lang, "tasks"),
                          planningTasks && e("span", { className: "pill", style: { marginLeft: "auto" } }, tr(lang, "planningTasks"))
                        ),
                        e(
                          "div",
                          { className: "task-list" },
                          sortedTasks.map((t) => {
                            const tgt = String(t.target || "coder") === "observer" ? "O" : "C";
                            const sendLabel = tgt === "O" ? tr(lang, "sendToObserver") : tr(lang, "sendToCoder");
                            const done = String(t.status || "") === "done";
                            return e(
                              "div",
                              { key: t.id, className: "task task-" + String(t.status || "new") },
                              e("div", { className: "task-head" },
                                e("span", { className: "task-target", title: String(t.target || "coder") }, tgt),
                                e("div", { className: "task-title", title: String(t.title || "") }, String(t.title || "(untitled)")),
                                t.phase && t.phase !== "any" && e("span", { className: "task-phase" }, String(t.phase)),
                                typeof t.priority === "number" && e("span", { className: "task-prio" }, String(t.priority) + "pt"),
                                e("span", { className: "task-status" }, String(t.status || "new")),
                                e("div", { className: "task-actions" },
                                  e("button", {
                                    className: "btn btn-primary",
                                    disabled: done,
                                    onClick: () => dispatchTask(t),
                                  }, sendLabel),
                                  e("button", {
                                    className: "btn",
                                    disabled: done,
                                    onClick: () => markTaskDone(t),
                                  }, tr(lang, "done"))
                                )
                              ),
                              t.body ? e("pre", { className: "task-body" }, String(t.body || "").trim()) : null
                            );
                          })
                        )
                      )
                    : null,
                  sortedProposals && sortedProposals.length
                    ? e(
                  "div",
                  { className: "proposalbox" },
                  e("div", { className: "section-title", style: { margin: 0 } }, tr(lang, "proposals")),
                  e(
                    "div",
                    { className: "proposal-list" },
                    sortedProposals.map((p) => {
                      const score = typeof p.score === "number" ? p.score : 50;
                      const phaseMismatch = observerPhase && p.phase && p.phase !== "any" && p.phase !== observerPhase;
                      const alreadyApproved = approvedProposalTitles && approvedProposalTitles.has(normTitle(p.title));
                      const scoreColor = score >= 70 ? "var(--accent)" : score >= 40 ? "var(--accent2)" : "var(--warn)";
                      const phaseLabel = p.phase && p.phase !== "any" ? p.phase : null;
                      const metaOp =
                        status && status.features && status.features.meta_prompts
                          ? parseMetaPromptOp(String(p.toCoder || ""))
                          : null;
                      return e(
                        "div",
                        { key: p.id, className: "proposal sev-" + p.severity + (phaseMismatch ? " phase-mismatch" : "") },
                        e("div", { className: "proposal-head" },
                          e("div", { style: { display: "flex", flexDirection: "column", gap: 4, flex: 1, minWidth: 0 } },
                            e("div", { style: { display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" } },
                              e("div", { className: "proposal-title" }, p.title),
                              phaseLabel && e("span", {
                                className: "proposal-phase-badge " + (phaseMismatch ? "phase-miss" : "phase-match"),
                                title: phaseMismatch ? `${tr(lang, "phaseMismatch")} (${observerPhase} -> ${p.phase})` : p.phase,
                              }, p.phase),
                              p.cost && e("span", { className: "cost-badge" }, p.cost),
                              p.status && p.status !== "new" && e("span", {
                                className: "status-badge status-" + (
                                  p.status.includes("UNRESOLVED") ? "unresolved" :
                                  p.status.includes("ESCALATED") ? "escalated" :
                                  p.status === "addressed" ? "addressed" : "info"
                                ),
                              }, p.status),
                              alreadyApproved && e("span", { className: "status-badge status-approved" }, tr(lang, "approved")),
                            ),
                            e("div", { className: "proposal-score-bar" },
                              e("div", { className: "proposal-score-fill", style: { width: score + "%", background: `linear-gradient(90deg, ${scoreColor}88, ${scoreColor})` } })
                            ),
                            e("div", { style: { display: "flex", gap: 8, alignItems: "center" } },
                              e("span", { className: "proposal-score-text" }, score + "pt"),
                              p.impact && e("span", { style: { fontSize: 11, color: "var(--muted)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" } }, p.impact),
                            ),
                            p.quote && p.quote !== "n/a" && e("div", {
                              style: { fontSize: 11, fontFamily: "var(--mono)", color: "rgba(45,212,191,0.7)",
                                       overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", marginTop: 2 },
                              title: p.quote,
                            }, "❝ " + p.quote),
                          ),
                          e("div", { className: "proposal-actions" },
                            score < 30 && e("span", { style: { fontSize: 10, color: "var(--faint)", fontFamily: "var(--mono)", whiteSpace: "nowrap" } }, "低優先"),
                            metaOp && e("button", {
                              className: "btn",
                              disabled: metaBusy,
                              onClick: () => applyMetaOp(metaOp),
                              title: "Apply to runtime prompt (requires approval)",
                            }, tr(lang, "applyMeta")),
                            e("button", {
                              className: "btn btn-primary",
                              disabled: alreadyApproved || sendingCoder || !String(p.toCoder || "").trim(),
                              onClick: () => {
                                if (alreadyApproved) { showToast(tr(lang, "alreadyApproved"), "info"); return; }
                                if (phaseMismatch && !confirmPhaseMismatch(lang, observerPhase, p.phase)) return;
                                setProposalModal(p);
                                setProposalModalText(String(p.toCoder || "").trim());
                              },
                              title: alreadyApproved
                                ? tr(lang, "alreadyApproved")
                                : (phaseMismatch ? `${tr(lang, "phaseMismatch")} (${observerPhase} -> ${p.phase})` : ""),
                            }, tr(lang, "sendToCoder"))
                          )
                        ),
                        p.toCoder ? e(
                          "div",
                          null,
                          e("button", {
                            className: "proposal-toggle",
                            onClick: () => setExpandedProposals((prev) => {
                              const next = new Set(prev);
                              if (next.has(p.id)) next.delete(p.id); else next.add(p.id);
                              return next;
                            }),
                          }, (expandedProposals.has(p.id) ? "▼ " + tr(lang, "hide") : "▶ " + tr(lang, "details"))),
                          expandedProposals.has(p.id)
                            ? e("pre", { className: "proposal-body" }, String(p.toCoder || "").trim())
                            : null
                        ) : null
                      );
                    })
                  )
                )
                  : null,
                  ), // end obs-scroll-zone
                  e(
                    "div",
                    { className: "composer" },
                    e("textarea", {
                      className: "textarea",
                      value: observerInput,
                      placeholder: tr(lang, "placeholder"),
                      rows: Math.max(2, Math.min(8, (observerInput.match(/\n/g) || []).length + 1)),
                      style: { resize: "none" },
                      onChange: (ev) => setObserverInput(ev.target.value),
                      onKeyDown: (ev) => {
                        if ((ev.metaKey || ev.ctrlKey) && ev.key === "Enter") {
                          ev.preventDefault();
                          sendObserver();
                          return;
                        }
                        if (ev.key === "Enter" && !ev.shiftKey) {
                          ev.preventDefault();
                          sendObserver();
                        }
                      },
                    }),
                    sendingObserver
                      ? e("button", { className: "btn btn-warn", onClick: stopObserver }, tr(lang, "stop"))
                      : e("button", { className: "btn btn-primary", onClick: sendObserver }, tr(lang, "send"))
                  ),
                  e(
                    "div",
                    { className: "statusline" },
                    e("span", {
                      className: "dot" + (sendingObserver ? " streaming" : ""),
                      style: {
                        background: PROVIDER_COLORS[observerResolvedProvider()] || (sendingObserver ? "rgba(251, 191, 36, 0.95)" : "rgba(251, 191, 36, 0.85)"),
                        transition: "background 400ms ease",
                      },
                    }),
                    e("span", null, sendingObserver ? (config.stream ? tr(lang, "streaming") : tr(lang, "sending")) : tr(lang, "ready")),
                    healthScore && e("span", {
                      className: "pill health-badge",
                      style: {
                        background: healthScore.score >= 70 ? "rgba(45,212,191,0.15)" : healthScore.score >= 40 ? "rgba(251,191,36,0.15)" : "rgba(251,113,133,0.15)",
                        borderColor: healthScore.score >= 70 ? "rgba(45,212,191,0.5)" : healthScore.score >= 40 ? "rgba(251,191,36,0.5)" : "rgba(251,113,133,0.5)",
                        color: healthScore.score >= 70 ? "var(--accent)" : healthScore.score >= 40 ? "#fbbf24" : "var(--warn)",
                      },
                      title: healthScore.rationale,
                    }, "❤ " + healthScore.score),
                    observerMsgs.length > 0 && e(
                      "span",
                      { style: { marginLeft: "auto", color: "var(--faint)", fontSize: 11, fontFamily: "var(--mono)" } },
                      "~" + observerMsgs.reduce((s, m) => s + estimateTokens(m.content), 0) + " tokens · " + observerMsgs.length + " msgs"
                    )
                  )
                ) // analysis Fragment
          )
        )
      ),
      e("div", { className: "toast-container" },
        toasts.map((t) => e("div", { key: t.id, className: "toast toast-" + t.type }, t.msg))
      ),

      // Proposal send modal
      proposalModal && e(
        "div",
        { className: "modal-overlay", onClick: () => setProposalModal(null) },
        e(
          "div",
          { className: "modal-box proposal-send-modal", onClick: (ev) => ev.stopPropagation() },
          e("div", { className: "modal-header" },
            e("div", { style: { display: "flex", flexDirection: "column", gap: 3 } },
              e("h3", null, String(proposalModal.title || "")),
              e("span", { style: { fontSize: 11, fontFamily: "var(--mono)", color: "var(--muted)" } },
                `[${String(proposalModal.severity || "info")}]` +
                (proposalModal.score != null ? ` · ${proposalModal.score}pt` : "")
              )
            ),
            e("button", { className: "btn btn-icon", title: tr(lang, "close"), onClick: () => setProposalModal(null) }, "×")
          ),
          e("textarea", {
            className: "textarea proposal-send-textarea",
            value: proposalModalText,
            onChange: (ev) => setProposalModalText(ev.target.value),
            spellCheck: false,
          }),
          e("div", { className: "modal-footer" },
            e("button", { className: "btn", onClick: () => setProposalModal(null) }, tr(lang, "close")),
            e("button", {
              className: "btn btn-primary",
              disabled: !proposalModalText.trim() || sendingCoder,
              onClick: () => {
                const to = proposalModalText.trim();
                if (!to) return;
                sendProposalToCoder({ ...proposalModal, toCoder: to });
                setProposalModal(null);
              },
            }, tr(lang, "sendToCoder"))
          )
        )
      ),

      // Reader modal (for readable long messages, especially Observer critiques)
      readerModal && e(
        "div",
        { className: "modal-overlay", onClick: () => setReaderModal(null) },
        e(
          "div",
          { className: "modal-box reader-modal", onClick: (ev) => ev.stopPropagation() },
          e("div", { className: "modal-header" },
            e("div", { style: { display: "flex", flexDirection: "column", gap: 3 } },
              e("h3", null, String(readerModal.title || tr(lang, "observer"))),
              readerModal.ts ? e("span", { style: { fontSize: 11, fontFamily: "var(--mono)", color: "var(--muted)" } }, relativeTime(readerModal.ts, lang)) : null
            ),
            e("button", { className: "btn btn-icon", title: tr(lang, "close"), onClick: () => setReaderModal(null) }, "×")
          ),
          e("div", { className: "reader-body" },
            renderWithThink(String(readerModal.content || ""), {}, null, null)
          ),
          e("div", { className: "modal-footer" },
            e("button", { className: "btn", onClick: () => setReaderModal(null) }, tr(lang, "close")),
            e("button", { className: "btn btn-primary", onClick: () => copyText(String(readerModal.content || "")) }, tr(lang, "copy"))
          )
        )
      ),

      // Keyboard shortcuts modal
      showShortcuts && e(
        "div",
        { className: "modal-overlay", onClick: () => setShowShortcuts(false) },
        e(
          "div",
          { className: "modal-box shortcuts-modal", onClick: (e) => e.stopPropagation() },
          e("div", { className: "modal-header" },
            e("h3", null, tr(lang, "shortcuts")),
            e("button", { className: "btn btn-icon", title: tr(lang, "close"), onClick: () => setShowShortcuts(false) }, "×")
          ),
          e(
            "table",
            { className: "shortcuts-table" },
            e("tbody", null,
              [
                ["Enter", tr(lang, "sendMsg")],
                ["Shift+Enter", tr(lang, "newline")],
                ["?", tr(lang, "toggleHelp")],
                ["Esc", tr(lang, "closeModals")],
                ["Ctrl+K", tr(lang, "stopStreamingCoder")],
              ].map(([key, desc]) =>
                e("tr", { key },
                  e("td", null, e("kbd", null, key)),
                  e("td", null, desc)
                )
              )
            )
          )
        )
      )
    );
  }

  // ╔══════════════════════════════════════════════════════════╗
  // ║  SECTION: Render                                         ║
  // ╚══════════════════════════════════════════════════════════╝
  try {
    if (ReactDOM.createRoot) {
      ReactDOM.createRoot(root).render(e(App));
    } else {
      ReactDOM.render(e(App), root);
    }
  } catch (err) {
    root.textContent = String(err && err.message ? err.message : err);
  }
})();
