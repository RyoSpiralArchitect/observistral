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
      includeCoderContext: "Include coder context",
      insertCliTemplate: "CLI template",
      editApproval: "Edit approval",
      commandApproval: "Command approval",
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
      apiKey: "API key",
      mode: "Mode",
      persona: "Persona",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "Streaming",
      cot: "CoT",
      brief: "Brief",
      structured: "Structured",
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
      mode: "モード",
      persona: "ペルソナ",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "ストリーミング",
      cot: "CoT",
      brief: "簡易",
      structured: "構造化",
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
      apiKey: "Clé API",
      mode: "Mode",
      persona: "Persona",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "Streaming",
      cot: "CoT",
      brief: "Bref",
      structured: "Structuré",
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

  const PROVIDERS = [
    { k: "mistral", l: { ja: "Mistral", en: "Mistral", fr: "Mistral" } },
    { k: "codestral", l: { ja: "Codestral", en: "Codestral", fr: "Codestral" } },
    { k: "mistral-cli", l: { ja: "Mistral CLI", en: "Mistral CLI", fr: "Mistral CLI" } },
    { k: "openai-compatible", l: { ja: "OpenAI互換", en: "OpenAI compat", fr: "Compat OpenAI" } },
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
    toolRoot: "",
    persona: "default",
    observerMode: "Observer",
    observerPersona: "novelist",
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

  function parseMarkdown(text) {
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
        out.push(e("div", { className: "code", key: k++ },
          e("div", { className: "code-head" },
            e("span", null, lang || "code"),
            e("button", {
              style: { marginLeft: "auto", background: "none", border: "none", color: "var(--muted)", cursor: "pointer", fontSize: "11px" },
              onClick: () => navigator.clipboard && navigator.clipboard.writeText(codeText).catch(() => {}),
            }, "⎘ copy"),
          ),
          e("pre", null, codeText),
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
      if (!title && !toCoder) {
        cur = null;
        lastKey = "";
        return;
      }
      out.push({
        id: `${out.length + 1}:${title || toCoder}`.slice(0, 80),
        title: title || "(untitled)",
        toCoder,
        severity: severity === "crit" || severity === "warn" ? severity : "info",
      });
      cur = null;
      lastKey = "";
    };

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const start = /^\s*(\d+)\)\s*title\s*:\s*(.*)\s*$/.exec(line);
      if (start) {
        finish();
        cur = { title: start[2], toCoder: "", severity: "info" };
        lastKey = "title";
        continue;
      }

      if (!cur) continue;

      const to = /^\s*to_coder\s*:\s*(.*)\s*$/.exec(line);
      if (to) {
        cur.toCoder = to[1];
        lastKey = "to_coder";
        continue;
      }

      const sev = /^\s*severity\s*:\s*(info|warn|crit)\b/i.exec(line);
      if (sev) {
        cur.severity = sev[1].toLowerCase();
        lastKey = "severity";
        continue;
      }

      // Continuation lines (indented) for multi-line fields.
      if (/^\s+/.test(line)) {
        const cont = line.trim();
        if (!cont) continue;
        if (lastKey === "to_coder") cur.toCoder = String(cur.toCoder || "") + "\n" + cont;
        if (lastKey === "title") cur.title = String(cur.title || "") + " " + cont;
      }
    }

    finish();
    return out;
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
      if (typeof cfg.includeCoderContext !== "boolean") cfg.includeCoderContext = !!DEFAULT_CONFIG.includeCoderContext;
      if (typeof cfg.requireEditApproval !== "boolean") cfg.requireEditApproval = !!DEFAULT_CONFIG.requireEditApproval;
      if (typeof cfg.requireCommandApproval !== "boolean") cfg.requireCommandApproval = !!DEFAULT_CONFIG.requireCommandApproval;
      if (typeof cfg.toolRoot !== "string") cfg.toolRoot = String(cfg.toolRoot || "");
      const cot0 = String(cfg.cot || "").trim().toLowerCase();
      if (cot0 === "off") cfg.cot = "off";
      else if (cot0 === "structured") cfg.cot = "structured";
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
    const [apiKey, setApiKey] = useState("");
    const [diff, setDiff] = useState("");
    const [models, setModels] = useState([]);
    const [modelsLoading, setModelsLoading] = useState(false);
    const [modelsErr, setModelsErr] = useState("");
    const [pendingEdits, setPendingEdits] = useState([]);
    const [pendingBusy, setPendingBusy] = useState(false);

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

    const abortCoderRef = useRef(null);
    const abortObserverRef = useRef(null);
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
        await postJson(approve ? "/api/approve_edit" : "/api/reject_edit", { id: eid });
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
    const keyAnthropic =
      status && status.providers && status.providers.anthropic ? !!status.providers.anthropic.api_key_present : false;
    const keyInputPresent = String(apiKey || "").trim().length > 0;
    const keyMistralOk = keyMistral || (config.provider === "mistral" && keyInputPresent);
    const keyCodestralOk = keyCodestral || (config.provider === "codestral" && keyInputPresent);
    const keyOpenAIOk = keyOpenAI || (config.provider === "openai-compatible" && keyInputPresent);
    const keyAnthropicOk = keyAnthropic || (config.provider === "anthropic" && keyInputPresent);

    const KeyDot = ({ ok }) => e("span", { className: "kdot " + (ok ? "ok" : "missing") });

    const providerLabel = (k) => {
      const p = PROVIDERS.find((x) => x.k === k);
      return (p && p.l && p.l[lang]) || k;
    };

    const modeLabel = (k) => {
      const m = MODES.find((x) => x.k === k);
      return (m && m.l && m.l[lang]) || k;
    };

    const modelForMode = (mode) => {
      const m0 = String(mode || "");
      const useCode = m0 === "VIBE" || m0.startsWith("diff") || m0 === "ログ解析";
      const m = useCode ? (config.codeModel || config.model) : (config.chatModel || config.model);
      return String(m || "").trim();
    };

    const coderActiveModel = () => modelForMode(config.mode);
    const observerActiveModel = () => modelForMode(config.observerMode);

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
        const j = await postJson("/api/models", {
          provider: config.provider,
          base_url: strOrUndef(config.baseUrl),
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
        observer_model_selected: observerActiveModel(),
      });
      const safe = String(activeThread.title || "thread").replace(/[\\/:*?\"<>|]/g, "_");
      downloadText(`obstral-${safe}.md`, md);
    };

    const copyText = async (text) => {
      try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          await navigator.clipboard.writeText(String(text || ""));
          return;
        }
      } catch (_) {}
      window.prompt(tr(lang, "copy"), String(text || ""));
    };

    const renderMessage = (m) =>
      e(
        "div",
        { key: m.id, className: "msg" },
        e("div", { className: "avatar" }, m.role === "user" ? "U" : "A"),
        e(
          "div",
          { className: "bubble " + (m.role === "user" ? "user" : "assistant") },
          e(
            "div",
            { className: "msg-meta" },
            e("div", { className: "who" }, m.role),
            e("div", { className: "mini" }, e("button", { onClick: () => copyText(m.content || "") }, tr(lang, "copy")))
          ),
          e("div", { className: "content" }, m.streaming ? String(m.content || "") : parseMarkdown(String(m.content || "")))
        )
      );

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

      const sendCoder = async (overrideText) => {
        if (sendingCoder) return;
        const raw = overrideText != null ? String(overrideText) : String(coderInput || "");
        const text = raw.trim();
        if (!text) return;

        const coderCfg = String(config.mode || "").trim() === "Observer" ? { ...config, mode: "VIBE" } : config;
        if (String(config.mode || "").trim() === "Observer") {
          setConfig((c) => ({ ...c, mode: "VIBE" }));
        }

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

      const reqBody = buildReq(coderCfg, apiKey, history, text, diff);
      reqBody.force_tools = true;
      const ac = new AbortController();
      abortCoderRef.current = ac;

      try {
        if (config.stream) {
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

    const sendObserver = async () => {
      if (sendingObserver) return;
      const text = String(observerInput || "").trim();
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
      setObserverInput("");
      setSendingObserver(true);
      requestAnimationFrame(() => scrollBottom(observerBodyRef));

      // Keep Observer in-character: do not inherit coder's CoT/autonomy formatting.
      const obsCfg = { ...config, mode: config.observerMode, persona: config.observerPersona, cot: "off", autonomy: "off" };
      const observerBridge = [
        "[Observer bridge]",
        "You are reviewing the coder's work artifacts.",
        "Use coder_context and code snippets to find bugs, risks, and missing tests.",
        "When you have actionable guidance, append a proposals block.",
      ].join("\n");
      const sendText = config.includeCoderContext
        ? (text + "\n\n" + observerBridge + "\n\n" + coderContextPacket())
        : (text + "\n\n" + observerBridge);
      const reqBody = buildReq(obsCfg, apiKey, history, sendText, diff);
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
            e("span", { className: "pill" }, status && status.version ? `v${status.version}` : "local"),
            e(
              "span",
              { className: "pill", title: tr(lang, "keys") },
              e(KeyDot, { ok: keyCodestralOk }),
              " C  ",
              e(KeyDot, { ok: keyMistralOk }),
              " M  ",
              e(KeyDot, { ok: keyOpenAIOk }),
              " O  ",
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
                { className: "field", style: { marginTop: "10px" } },
                e("label", null, tr(lang, "apiKey")),
                e("input", {
                  className: "input",
                  type: "password",
                  value: apiKey,
                  onChange: (ev) => setApiKey(ev.target.value),
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
                    background: PROVIDER_COLORS[config.provider] || "rgba(255,255,255,0.4)",
                    boxShadow: "0 0 8px " + (PROVIDER_COLORS[config.provider] || "transparent") + "88",
                    transition: "background 400ms ease, box-shadow 400ms ease",
                  },
                }),
                e(
                  "span",
                  { className: "pill", title: coderActiveModel() },
                  providerLabel(config.provider) + " · " + modeLabel(config.mode) + " · " + shortModel(coderActiveModel())
                )
              )
            ),
            e("div", { className: "chat-body", ref: coderBodyRef }, coderMsgs.map(renderMessage)),
            e(
              "div",
              { className: "composer" },
              e("textarea", {
                className: "textarea",
                value: coderInput,
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
                className: "dot",
                style: {
                  background: sendingCoder
                    ? (PROVIDER_COLORS[config.provider] || "rgba(45,212,191,0.85)")
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
                    background: "rgba(251, 191, 36, 0.85)",
                    boxShadow: "0 0 8px rgba(251, 191, 36, 0.45)",
                    transition: "background 400ms ease, box-shadow 400ms ease",
                  },
                }),
                e(
                  "span",
                  { className: "pill", title: observerActiveModel() },
                  providerLabel(config.provider) + " · " + modeLabel(config.observerMode) + " · " + shortModel(observerActiveModel())
                )
              )
            ),
            e("div", { className: "chat-body", ref: observerBodyRef }, observerMsgs.map(renderMessage)),
            observerProposals && observerProposals.length
              ? e(
                  "div",
                  { className: "proposalbox" },
                  e("div", { className: "section-title", style: { margin: 0 } }, tr(lang, "proposals")),
                  e(
                    "div",
                    { className: "proposal-list" },
                    observerProposals.map((p) =>
                      e(
                        "div",
                        { key: p.id, className: "proposal sev-" + p.severity },
                        e("div", { className: "proposal-head" },
                          e("div", { className: "proposal-title" }, p.title),
                          e("div", { className: "proposal-actions" },
                            e(
                              "button",
                              {
                                className: "btn btn-primary",
                                disabled: sendingCoder || !String(p.toCoder || "").trim(),
                                onClick: () => sendProposalToCoder(p),
                              },
                              tr(lang, "sendToCoder")
                            )
                          )
                        ),
                        p.toCoder ? e("pre", { className: "proposal-body" }, String(p.toCoder || "").trim()) : null
                      )
                    )
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
                className: "dot",
                style: {
                  background: sendingObserver ? "rgba(251, 191, 36, 0.95)" : "rgba(251, 191, 36, 0.85)",
                  transition: "background 400ms ease",
                },
              }),
              e("span", null, sendingObserver ? (config.stream ? tr(lang, "streaming") : tr(lang, "sending")) : tr(lang, "ready")),
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
