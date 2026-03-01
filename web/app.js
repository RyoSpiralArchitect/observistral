(() => {
  "use strict";

  const root = document.getElementById("app-root");
  if (!root) return;

  const e = React.createElement;
  const { useEffect, useMemo, useRef, useState } = React;

  // SECTION: constants
  const LS = {
    lang: "obstral.lang.v1",
    config: "obstral.config.v1",
    threads: "obstral.threads.v1",
    active: "obstral.active.v1",
  };

  // SECTION: i18n (filled below)
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
      sendToCoder: "Send to coder",
      applyMeta: "Apply",
      includeCoderContext: "Include coder context",
      insertCliTemplate: "CLI template",
      editApproval: "Edit approval",
      commandApproval: "Command approval",
      autoObserve: "Auto-observe",
      forceAgent: "Agent mode",
      toolRoot: "作業ルート",
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
      coder: "Coder",
      observer: "Observer",
      proposals: "提案",
      sendToCoder: "Coderへ送る",
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
      forceAgent: "エージェント常時ON",
      toolRoot: "Tool root",
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
      sendToCoder: "Envoyer au codeur",
      applyMeta: "Appliquer",
      includeCoderContext: "Inclure contexte codeur",
      insertCliTemplate: "Template CLI",
      editApproval: "Approbation édition",
      commandApproval: "Approbation commande",
      toolRoot: "Racine outils",
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
      forceAgent: "Mode agent (toujours)",
      serverOutdated: "Serveur obsolète",
    },
  };

  function tr(lang, key) {
    return (I18N[lang] && I18N[lang][key]) || I18N.en[key] || key;
  }

  // SECTION: utils
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

  function normalizeForSim(s) {
    let t = String(s || "");
    // Remove fenced code blocks and inline code to reduce false positives.
    t = t.replace(/```[\s\S]*?```/g, " ");
    t = t.replace(/`[^`]*`/g, " ");
    t = t.toLowerCase();
    t = t.replace(/https?:\/\/\S+/g, " ");
    // Keep latin, extended latin, numbers, and common CJK ranges.
    t = t.replace(/[^a-z0-9\u00c0-\u024f\u3040-\u30ff\u3400-\u9fff]+/g, " ");
    t = t.replace(/\s+/g, " ").trim();
    return t;
  }

  function tokenSetForSim(s) {
    const t = normalizeForSim(s);
    const out = new Set();
    if (!t) return out;

    // Word-ish tokens.
    t.split(" ").forEach((w) => {
      const x = String(w || "").trim();
      if (x.length >= 2) out.add(x);
    });

    // For CJK-heavy text, add bigrams so we can detect repetition even without spaces.
    const cjk = t.replace(/[a-z0-9\u00c0-\u024f ]+/g, "");
    if (cjk.length >= 8) {
      for (let i = 0; i < cjk.length - 1 && out.size < 2400; i++) {
        out.add(cjk.slice(i, i + 2));
      }
    }
    return out;
  }

  function jaccardSim(aSet, bSet) {
    if (!aSet || !bSet || !aSet.size || !bSet.size) return 0;
    let inter = 0;
    for (const x of aSet) if (bSet.has(x)) inter++;
    const union = aSet.size + bSet.size - inter;
    return union ? inter / union : 0;
  }

  function similarity(a, b) {
    return jaccardSim(tokenSetForSim(a), tokenSetForSim(b));
  }

  function autoObservePrompt(uiLang) {
    const l = String(uiLang || "").trim().toLowerCase();
    if (l === "fr") {
      return "[AUTO-OBSERVE] Le Coder vient de produire une nouvelle sortie. Fais une critique en mode commentaire live.\n"
        + "1) Commence par UNE phrase: qu'est-ce qui vient d'arriver.\n"
        + "2) Analyse les risques sur 5 axes: exactitude, sécurité, fiabilité, performance, maintenabilité.\n"
        + "3) Termine par un bloc --- proposals --- avec des actions concrètes.";
    }
    if (l === "en") {
      return "[AUTO-OBSERVE] The Coder produced new output. Commentate and critique live.\n"
        + "1) Start with ONE sentence: what just happened.\n"
        + "2) Risk scan across 5 axes: correctness, security, reliability, performance, maintainability.\n"
        + "3) End with a --- proposals --- block with concrete actions.";
    }
    return "[AUTO-OBSERVE] コーダーが新しいアウトプットを生成した。実況しながら批評せよ。\n"
      + "1) まず一文で「何が起きたか」を述べる。\n"
      + "2) 5軸（正しさ/セキュリティ/信頼性/性能/保守性）でリスクを洗い出す。\n"
      + "3) 最後に --- proposals --- ブロックで具体的な手を出す。";
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

  const PERSONAS = ["default", "novelist", "cynical", "cheerful", "thoughtful"];

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
    toolRoot: "",
    mistralCliAgent: "",
    mistralCliMaxTurns: "",
    codeProvider: "",
    codeBaseUrl: "",
    observerProvider: "",
    observerBaseUrl: "",
    observerModel: "",
    persona: "default",
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
    return m && m.pane === "observer" ? "O" : "C";
  }

  function whoLabel(m, lang) {
    const L = String(lang || "ja").trim().toLowerCase();
    if (m && m.role === "user") return L === "fr" ? "Vous" : L === "en" ? "You" : "あなた";
    return m && m.pane === "observer" ? "Observer" : "Coder";
  }

  function makeThread(title) {
    return {
      id: uid(),
      title: title || "Untitled",
      createdAt: Date.now(),
      updatedAt: Date.now(),
      messages: [],
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

  // SECTION: api (filled below)
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

  // SECTION: provider colors
  const PROVIDER_COLORS = {
    "mistral":           "#2dd4bf",
    "codestral":         "#14b8a6",
    "mistral-cli":       "#34d399",
    "openai-compatible": "#60a5fa",
    "gemini":            "#a3e635",
    "anthropic":         "#fb7185",
    "hf":                "#fbbf24",
  };

  // SECTION: markdown renderer
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

  function isWindowsHost() {
    try {
      return typeof navigator !== "undefined" && /Windows/i.test(String(navigator.userAgent || ""));
    } catch (_) {
      return false;
    }
  }

  function stripShellTranscript(text) {
    const raw = String(text || "").replace(/\r\n/g, "\n");
    const lines = raw.split("\n");
    const promptRe = /^\s*(\$\s+|PS(?: [^>]+)?>\s+|>\s+)/;
    const hasPrompt = lines.some((l) => promptRe.test(l));
    const out = [];
    for (const l0 of lines) {
      const l = String(l0 || "");
      if (hasPrompt) {
        if (!promptRe.test(l)) continue; // drop output lines
        out.push(l.replace(promptRe, ""));
      } else {
        out.push(l.replace(/^\s*\$\s+/, ""));
      }
    }
    return out.join("\n").trim();
  }

  function dangerousCommandReason(cmd) {
    const s0 = String(cmd || "");
    const s = s0.toLowerCase().replace(/\s+/g, " ").trim();
    if (!s) return "";
    if (s.indexOf("git reset --hard") !== -1) return "git reset --hard";
    if (/\bgit\s+clean\b/.test(s) && /\b-[a-z]*f[a-z]*\b/.test(s) && /\b-[a-z]*d[a-z]*\b/.test(s)) return "git clean -fd";
    if (/\bgit\s+rm\b/.test(s) && /\b(--cached|-r|--recursive)\b/.test(s) && /(^|\\s)\\.(\\s|$)/.test(s)) return "git rm ... .";
    if (/\brm\s+-rf\b/.test(s)) return "rm -rf";
    if (s.indexOf("remove-item") !== -1 && s.indexOf("-recurse") !== -1 && s.indexOf("-force") !== -1) return "Remove-Item -Recurse -Force";
    return "";
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

  function splitArgsSimple(s) {
    const src = String(s || "");
    const out = [];
    let cur = "";
    let q = null;
    for (let i = 0; i < src.length; i++) {
      const ch = src[i];
      if (q) {
        if (ch === q) { q = null; continue; }
        if (ch === "\\" && q === "\"" && i + 1 < src.length) { cur += src[i + 1]; i++; continue; }
        cur += ch;
        continue;
      }
      if (ch === "'" || ch === "\"") { q = ch; continue; }
      if (/\s/.test(ch)) {
        if (cur) { out.push(cur); cur = ""; }
        continue;
      }
      cur += ch;
    }
    if (cur) out.push(cur);
    return out;
  }

  function psSingleQuote(s) {
    return "'" + String(s || "").replace(/'/g, "''") + "'";
  }

  function bashToPowerShell(script) {
    const raw = stripShellTranscript(script);
    if (!raw) return "";
    const lines = raw.split("\n");
    const out = [];
    out.push("$ErrorActionPreference = 'Stop'");

    for (const line0 of lines) {
      let line = String(line0 || "").trim();
      if (!line) continue;
      if (line.startsWith("#")) continue;

      // Common model glitch: trailing brace.
      if (line.endsWith("}") && line.indexOf("{") === -1) line = line.slice(0, -1).trim();

      line = line.replace(/&&/g, ";");
      const segs = line.split(";"); // naive but good enough for typical command snippets
      for (const seg0 of segs) {
        const seg = String(seg0 || "").trim();
        if (!seg) continue;

        const toks = splitArgsSimple(seg);
        if (!toks.length) continue;

        const head = toks[0];
        if (head === "mkdir") {
          let parents = false;
          const paths = [];
          for (let i = 1; i < toks.length; i++) {
            const t = toks[i];
            if (t === "-p" || t === "--parents") { parents = true; continue; }
            if (t && t.startsWith("-")) continue;
            paths.push(t);
          }
          if (parents) {
            for (const p of paths) out.push("[System.IO.Directory]::CreateDirectory(" + psSingleQuote(p) + ") | Out-Null");
          } else {
            out.push(seg); // mkdir is an alias in PowerShell
          }
          continue;
        }

        if (head === "touch") {
          const paths = [];
          for (let i = 1; i < toks.length; i++) {
            const t = toks[i];
            if (t && t.startsWith("-")) continue;
            paths.push(t);
          }
          for (const p of paths) {
            const q = psSingleQuote(p);
            out.push("if (-not (Test-Path -LiteralPath " + q + ")) { New-Item -ItemType File -Force -Path " + q + " | Out-Null } else { (Get-Item -LiteralPath " + q + ").LastWriteTime = Get-Date }");
          }
          continue;
        }

        if (head === "rm") {
          let recurse = false;
          const paths = [];
          for (let i = 1; i < toks.length; i++) {
            const t = toks[i];
            if (t && t.startsWith("-")) { if (t.indexOf("r") !== -1 || t.indexOf("R") !== -1) recurse = true; continue; }
            paths.push(t);
          }
          for (const p of paths) {
            const q = psSingleQuote(p);
            out.push("Remove-Item -Force " + (recurse ? "-Recurse " : "") + "-ErrorAction SilentlyContinue -LiteralPath " + q);
          }
          continue;
        }

        if (head === "cp" && toks.length >= 3) {
          const src = psSingleQuote(toks[1]);
          const dst = psSingleQuote(toks[2]);
          out.push("Copy-Item -Force -LiteralPath " + src + " -Destination " + dst);
          continue;
        }

        if (head === "mv" && toks.length >= 3) {
          const src = psSingleQuote(toks[1]);
          const dst = psSingleQuote(toks[2]);
          out.push("Move-Item -Force -LiteralPath " + src + " -Destination " + dst);
          continue;
        }

        if (head === "export" && toks.length >= 2 && toks[1].indexOf("=") !== -1) {
          const idx = toks[1].indexOf("=");
          const k = toks[1].slice(0, idx);
          const v = toks[1].slice(idx + 1);
          if (k) out.push("$env:" + k + " = " + psSingleQuote(v));
          continue;
        }

        out.push(seg);
      }
    }

    return out.join("\n").trim();
  }

  function normalizeExecScript(lang, codeText) {
    const isWin = isWindowsHost();
    const l = String(lang || "").trim().toLowerCase();
    const cleaned = stripShellTranscript(codeText);
    if (!isWin) return cleaned;

    const isBash = /^(bash|sh|shell|zsh|console)$/.test(l);
    const isPwsh = /^(powershell|pwsh|ps1|ps)$/.test(l);
    if (isPwsh) return cleaned;
    if (isBash) return bashToPowerShell(cleaned);

    // Tool logs often use ```bash``` even on Windows.
    if (/(^|\s)(mkdir\s+-p\b|touch\b|rm\s+-rf\b)/.test(cleaned) || cleaned.indexOf("&&") !== -1) {
      return bashToPowerShell(cleaned);
    }
    return cleaned;
  }

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

  function parseMarkdown(text, execRes, onRun) {
    const lines = text.split("\n");
    const out = [];
    let i = 0, k = 0;
    while (i < lines.length) {
      const line = lines[i];
      if (line.startsWith("```")) {
        const lang = line.slice(3).trim();
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
            e("span", null, lang || "code"),
            isRunnable && e("button", {
              className: "btn-run",
              disabled: !!(res && res.running),
              onClick: () => onRun(blockIdx, lang, codeText),
            }, res && res.running ? "⏳" : "▶ run"),
            e("button", {
              style: { marginLeft: isRunnable ? "0" : "auto", background: "none", border: "none", color: "var(--muted)", cursor: "pointer", fontSize: "11px" },
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

  // Renders message content with <think>…</think> blocks dimmed separately.
  function renderWithThink(text, execRes, onRun) {
    const re = /<think>([\s\S]*?)<\/think>/gi;
    const parts = [];
    let last = 0;
    let k = 0;
    let m;
    while ((m = re.exec(text)) !== null) {
      if (m.index > last) {
        parts.push(...parseMarkdown(text.slice(last, m.index), execRes, onRun));
      }
      parts.push(e("div", { key: "think" + k++, className: "think-block" }, m[1].trim()));
      last = m.index + m[0].length;
    }
    if (last < text.length) {
      parts.push(...parseMarkdown(text.slice(last), execRes, onRun));
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

  function parseProposals(text) {
    const s = String(text || "");
    const m = /---\s*proposals\s*---/i.exec(s);
    if (!m) return [];

    const tail = s.slice(m.index + m[0].length);
    const lines = tail.split(/\r?\n/);

    const out = [];
    let cur = null;
    let lastKey = "";

    const finish = () => {
      if (!cur) return;
      const title = String(cur.title || "").trim();
      const toCoder = String(cur.toCoder || "").trim();
      const severity = String(cur.severity || "info").trim().toLowerCase();
      if (!title && !toCoder) { cur = null; lastKey = ""; return; }
      const rawScore = parseInt(String(cur.score || ""), 10);
      out.push({
        id: `${out.length + 1}:${title || toCoder}`.slice(0, 80),
        title: title || "(untitled)",
        toCoder,
        severity: severity === "crit" || severity === "warn" ? severity : "info",
        score: Number.isFinite(rawScore) ? Math.min(100, Math.max(0, rawScore)) : 50,
        phase: String(cur.phase || "any").trim().toLowerCase(),
        impact: String(cur.impact || "").trim(),
        cost: String(cur.cost || "").trim().toLowerCase(),
      });
      cur = null;
      lastKey = "";
    };

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      // Stop if we hit another --- block
      if (/^\s*---/.test(line) && out.length === 0 && !cur) continue;
      if (/^\s*---/.test(line) && (out.length > 0 || cur)) { finish(); break; }

      const start = /^\s*(\d+)\)\s*title\s*:\s*(.*)\s*$/.exec(line);
      if (start) { finish(); cur = { title: start[2], toCoder: "", severity: "info" }; lastKey = "title"; continue; }

      if (!cur) continue;

      const to = /^\s*to_coder\s*:\s*(.*)\s*$/.exec(line);
      if (to) { cur.toCoder = to[1]; lastKey = "to_coder"; continue; }

      const sev = /^\s*severity\s*:\s*(info|warn|crit)\b/i.exec(line);
      if (sev) { cur.severity = sev[1].toLowerCase(); lastKey = "severity"; continue; }

      const sc = /^\s*score\s*:\s*(\d+)/.exec(line);
      if (sc) { cur.score = sc[1]; lastKey = "score"; continue; }

      const ph = /^\s*phase\s*:\s*(\w+)/.exec(line);
      if (ph) { cur.phase = ph[1]; lastKey = "phase"; continue; }

      const imp = /^\s*impact\s*:\s*(.+)$/.exec(line);
      if (imp) { cur.impact = imp[1]; lastKey = "impact"; continue; }

      const co = /^\s*cost\s*:\s*(\w+)/.exec(line);
      if (co) { cur.cost = co[1]; lastKey = "cost"; continue; }

      const st = /^\s*status\s*:\s*(.+)$/.exec(line);
      if (st) { cur.status = st[1].trim(); lastKey = "status"; continue; }

      const qt = /^\s*quote\s*:\s*(.+)$/.exec(line);
      if (qt) { cur.quote = qt[1].trim(); lastKey = "quote"; continue; }

      // Continuation lines (indented)
      if (/^\s+/.test(line)) {
        const cont = line.trim();
        if (!cont) continue;
        if (lastKey === "to_coder") cur.toCoder = String(cur.toCoder || "") + "\n" + cont;
        if (lastKey === "title") cur.title = String(cur.title || "") + " " + cont;
        if (lastKey === "impact") cur.impact = String(cur.impact || "") + " " + cont;
      }
    }

    finish();
    return out;
  }

  function parseCriticalPath(text) {
    const s = String(text || "");
    const m = /---\s*critical_path\s*---/i.exec(s);
    if (!m) return "";
    const after = s.slice(m.index + m[0].length).trimStart();
    const line = after.split(/\r?\n/)[0].trim();
    return line.toLowerCase() === "none" || !line ? "" : line;
  }

  function parseHealthScore(text) {
    const s = String(text || "");
    const m = /---\s*health\s*---/i.exec(s);
    if (!m) return null;
    const block = s.slice(m.index + m[0].length);
    const sc = /score\s*:\s*(\d+)/.exec(block);
    const ra = /rationale\s*:\s*(.+)/.exec(block);
    if (!sc) return null;
    return {
      score: Math.min(100, Math.max(0, parseInt(sc[1], 10))),
      rationale: ra ? ra[1].trim() : "",
    };
  }

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

  // SECTION: app (filled below)
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
          messages: Array.isArray(t.messages)
            ? t.messages
                .filter((m) => m && typeof m === "object")
                .map((m) => ({
                  id: typeof m.id === "string" && m.id ? m.id : uid(),
                  pane: m.pane === "observer" ? "observer" : "coder",
                  role: m.role === "assistant" ? "assistant" : "user",
                  content: typeof m.content === "string" ? m.content : String(m.content || ""),
                  ts: typeof m.ts === "number" ? m.ts : Date.now(),
                  streaming: !!m.streaming,
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
    const [sendingCoder, setSendingCoder] = useState(false);
    const [sendingObserver, setSendingObserver] = useState(false);
    const [loopInfo, setLoopInfo] = useState({ active: false, score: 0, depth: 0 });
    const [execResults, setExecResults] = useState({}); // { msgId: { blockIdx: {stdout,stderr,exit_code,running} } }
    const [expandedProposals, setExpandedProposals] = useState(new Set());
    const [copiedId, setCopiedId] = useState(null);
    const [toasts, setToasts] = useState([]);

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

    const refreshStatus = () => {
      fetch("/api/status")
        .then((r) => r.json())
        .then((j) => setStatus(j))
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
      const next = window.prompt(tr(lang, "rename"), th ? th.title : "");
      if (!next || !next.trim()) return;
      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) => (t.id === id ? { ...t, title: next.trim(), updatedAt: Date.now() } : t)),
      }));
    };

    const deleteThread = (id) => {
      if (!window.confirm(tr(lang, "delConfirm"))) return;
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
        const cwd = String(config.toolRoot || "").trim() || undefined;
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

    const renderMessage = (m) => {
      const canExec = !!(status && status.features && status.features.exec);
      const s = String(m && m.content ? m.content : "");
      const choices = (!m.streaming && m.role === "assistant" && m.pane !== "observer") ? extractChoices(s) : [];
      const streamingNode = e("span", null,
        s || e("span", { className: "thinking" }, tr(lang, "streaming")),
        e("span", { className: "cursor-blink" }, "▊")
      );
      return e(
        "div",
        { key: m.id, className: "msg" + (m.role === "user" ? " msg-user" : " msg-assistant") },
        e("div", { className: "avatar" }, avatarLabel(m)),
        e(
          "div",
          { className: "bubble " + (m.role === "user" ? "user" : "assistant") },
          e(
            "div",
            { className: "msg-meta" },
            e("div", { className: "who" }, whoLabel(m, lang)),
            m.ts ? e("span", { className: "msg-ts" }, relativeTime(m.ts, lang)) : null,
            e("div", { className: "mini" }, e("button", {
              className: copiedId === m.id ? "copied" : "",
              onClick: () => copyText(m.content || "", m.id),
            }, copiedId === m.id ? "✓" : tr(lang, "copy")))
          ),
          e(
            "div",
            { className: "content" },
            m.streaming ? streamingNode : renderWithThink(
              String(m.content || ""),
              execResults[m.id] || {},
              canExec ? ((blockIdx, langHint, codeText) => runCmd(m.id, blockIdx, langHint, codeText)) : null
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
          )
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
      return all.filter((m) => m.pane !== "observer");
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
      parts.push(`tool_root: ${String(config.toolRoot || "").trim()}`);
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

    const runCoderAgentic = async (text, threadId, asstMsgId, reqCfg, resolvedKey, history, ac) => {
      const MAX_ITERS = 10;
      const TRUNC_STDOUT = 2000;
      const TRUNC_STDERR = 800;
      const KEEP_TOOL_TURNS = 4;

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
          const lines = content.split("\n");
          if (lines.length > 2) {
            msgs[idx] = { ...msgs[idx], content: lines[0] + ` [pruned ${lines.length}L]` };
          }
        }
      };

      const cwd = String(config.toolRoot || "").trim() || undefined;

      const isWindows = navigator.userAgent.includes("Windows");
      const SYSTEM_BASE = isWindows ? [
        "You are an autonomous coding agent with DIRECT access to the user's Windows machine.",
        "CRITICAL RULES — follow these without exception:",
        "1. ALWAYS use the `exec` tool to create files, directories, and run commands. NEVER just show code.",
        "2. Use PowerShell syntax ONLY (cmd.exe is NOT used):",
        "   - Create directory tree: New-Item -ItemType Directory -Force -Path 'a/b/c'",
        "   - Create file with content: Set-Content -Path 'file.txt' -Value 'line1`nline2' -Encoding UTF8",
        "   - Multi-line file: $content = @'\nline1\nline2\n'@; Set-Content -Path 'file.txt' -Value $content -Encoding UTF8",
        "   - Append to file: Add-Content -Path 'file.txt' -Value 'more' -Encoding UTF8",
        "   - Git: git init, git add ., git commit -m 'init'",
        "   - NEVER use mkdir -p, touch, cat >, or any Unix syntax.",
        "3. Execute ALL steps immediately via exec. Do NOT ask for permission or confirmation.",
        "4. After each exec call, read the output and continue until the task is 100% complete.",
        "5. End with a brief summary listing every file created/modified and any remaining steps.",
      ].join("\n") : [
        "You are an autonomous coding agent with DIRECT access to the user's local machine.",
        "CRITICAL RULES — follow these without exception:",
        "1. ALWAYS use the `exec` tool to create files, directories, and run commands. NEVER just show code.",
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
        "next: <≤12 words: exact command or step>",
        "verify: <≤12 words: how to confirm this step succeeded>",
        "</think>",
        "This 4-line check (~40 tokens) prevents wrong-direction errors that cost 300+ tokens to recover.",
        "",
        "[Error Protocol]",
        "If exit_code ≠ 0: STOP immediately.",
        "  1. Quote the exact error line.",
        "  2. State root cause in one sentence.",
        "  3. Fix with one corrected command.",
        "If the SAME approach fails 3 consecutive times: STOP, explain why,",
        "  and propose a completely different strategy. Never repeat a failing command.",
      ].join("\n");

      const SYSTEM = SYSTEM_BASE + SYSTEM_REASONING;

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

      const messages = [
        { role: "system", content: SYSTEM },
        ...history,
        { role: "user", content: text },
      ];

      let display = "";
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

      for (let iter = 0; iter < MAX_ITERS; iter++) {
        if (ac.signal.aborted) break;
        pruneToolMessages(messages);

        let streamResult;
        try {
          // Separate text tokens from previous turn with a blank line.
          if (display) display += "\n\n";
          streamResult = await streamChatTools({
            messages,
            tools: [execTool],
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
            if (tc.type !== "function" || tc.function.name !== "exec") continue;

            let args;
            try { args = JSON.parse(tc.function.arguments || "{}"); } catch (_) { args = {}; }
            const command = stripShellTranscript(String(args.command || ""));
            const commandToRun = normalizeExecScript("", command);
            const danger = dangerousCommandReason(commandToRun);

            const win = isWindowsHost();
            const fenceLang = win ? "powershell" : "bash";
            const prompt = win ? "PS> " : "$ ";
            display += (display ? "\n\n" : "") + "```" + fenceLang + "\n" + prompt + command;
            flush("\n```");

            let toolResult;
            try {
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

              const execRes = await postJson("/api/exec", { command: commandToRun, cwd }, ac.signal);
              const stdout = truncTool(execRes.stdout, TRUNC_STDOUT);
              const stderr = truncTool(execRes.stderr, TRUNC_STDERR);
              const exitCode = execRes.exit_code;
              const failed = exitCode !== 0 || (stderr && !stdout);

              toolResult = failed
                ? `FAILED (exit_code: ${exitCode}).\nstderr: ${stderr || "(empty)"}\nstdout: ${stdout || "(empty)"}\n⚠ The command failed. Diagnose the error above and call exec again with the fix. Do NOT continue to the next step until this succeeds.`
                : `OK (exit_code: 0)\nstdout: ${stdout || "(empty)"}`;

              if (stdout) display += "\n" + stdout;
              if (stderr) display += "\nstderr: " + stderr;
              display += "\n```\nexit: " + exitCode;
            } catch (execErr) {
              toolResult = `error: ${execErr.message}`;
              display += "\nerror: " + execErr.message + "\n```";
            }
            flush();

            messages.push({ role: "tool", tool_call_id: tc.id, content: toolResult });
          }
        } else {
          break; // finish_reason === "stop" — done
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

      const reqBody = buildReq(reqCfg, resolvedKey, history, text, diff);
      reqBody.lang = lang;
      reqBody.force_tools = !!(useCode || wantsMaterial);
      const ac = new AbortController();
      abortCoderRef.current = ac;

      try {
        const supportsTools = resolvedProvider === "openai-compatible" || resolvedProvider === "mistral" || resolvedProvider === "openai";
        const serverChatTools = !!(status && status.features && status.features.chat_tools);
        if ((config.forceAgent || wantsMaterial) && supportsTools && serverChatTools) {
          await runCoderAgentic(text, threadId, asstMsg.id, reqCfg, resolvedKey, history, ac);
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
      const langLine = outLang === "fr"
        ? "Langue: français. Écris la critique en français. Garde les clés du bloc proposals en anglais (title/to_coder/severity/score/phase/impact/cost)."
        : outLang === "en"
          ? "Language: English. Write the critique in English. Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost)."
          : "言語: 日本語。批評は日本語で書いてください。proposalsブロックのキー(title/to_coder/severity/score/phase/impact/cost)は英語のままにしてください。";
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
      const reqBody = buildReq(obsCfg, obsKey, history, sendText, diff);
      reqBody.lang = lang;
      reqBody.force_tools = false;
      const ac = new AbortController();
      abortObserverRef.current = ac;

      try {
        if (config.stream) {
          await streamChat(
            reqBody,
            (evt) => {
              if (!evt) return;
              if (evt.event === "delta") {
                const j = safeJsonParse(evt.data || "{}", {});
                if (j && j.delta) appendDelta(threadId, asstMsg.id, j.delta, observerBodyRef);
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
          setMsg(threadId, asstMsg.id, String((j && j.content) || ""), observerBodyRef);
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
            e("button", { className: "btn", onClick: refreshStatus, type: "button" }, tr(lang, "refresh"))
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
                  return e(
                    "div",
                    { key: t.id, style: { display: "flex", gap: "8px", alignItems: "stretch" } },
                    e(
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
                        `${new Date(t.updatedAt).toLocaleString()} · ${(t.messages || []).length} msgs`
                      )
                    ),
                    e("button", { className: "btn", style: { padding: "8px 10px" }, onClick: () => renameThread(t.id) }, "✎"),
                    e(
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
          { className: "arena" },
          // Coder pane
          e(
            "div",
            { className: "panel chat" },
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
            e("div", { className: "chat-body", ref: coderBodyRef },
              coderMsgs.length === 0
                ? e("div", { className: "pane-empty" },
                    e("div", { className: "pane-empty-icon" }, "⚡"),
                    e("p", { className: "pane-empty-hint" }, tr(lang, "placeholder"))
                  )
                : coderMsgs.map(renderMessage)
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

          // Observer pane
          e(
            "div",
            { className: "panel chat" },
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
            e("div", { className: "chat-body", ref: observerBodyRef },
              observerMsgs.length === 0
                ? e("div", { className: "pane-empty" },
                    e("div", { className: "pane-empty-icon" }, "👁"),
                    e("p", { className: "pane-empty-hint" }, tr(lang, "autoObserve"))
                  )
                : observerMsgs.map(renderMessage)
            ),
            criticalPath
              ? e("div", { className: "critical-path-banner" },
                  e("span", { className: "critical-path-icon" }, "⚠"),
                  e("span", { className: "critical-path-text" }, criticalPath)
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
                                title: phaseMismatch ? `フェーズ外 (現在: ${observerPhase}、提案: ${p.phase})` : p.phase,
                              }, p.phase),
                              p.cost && e("span", { className: "cost-badge" }, p.cost),
                              p.status && p.status !== "new" && e("span", {
                                className: "status-badge status-" + (
                                  p.status.includes("UNRESOLVED") ? "unresolved" :
                                  p.status.includes("ESCALATED") ? "escalated" :
                                  p.status === "addressed" ? "addressed" : "info"
                                ),
                              }, p.status),
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
                              disabled: sendingCoder || !String(p.toCoder || "").trim(),
                              onClick: () => sendProposalToCoder(p),
                              title: phaseMismatch ? `フェーズ外 (現在: ${observerPhase}、提案: ${p.phase})` : "",
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
                {
                  style: { marginLeft: "auto", color: "var(--faint)", fontSize: 11, fontFamily: "var(--mono)" },
                },
                "~" + observerMsgs.reduce((s, m) => s + estimateTokens(m.content), 0) + " tokens · " + observerMsgs.length + " msgs"
              )
            )
          )
        )
      ),
      e("div", { className: "toast-container" },
        toasts.map((t) => e("div", { key: t.id, className: "toast toast-" + t.type }, t.msg))
      )
    );
  }

  // SECTION: render
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
