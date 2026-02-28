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
      send: "Send",
      stop: "Stop",
      exportMd: "Markdown",
      clear: "Clear",
      provider: "Provider",
      model: "Model",
      chatModel: "Chat model",
      codeModel: "Code model",
      baseUrl: "Base URL",
      apiKey: "API key",
      mode: "Mode",
      persona: "Persona",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "Streaming",
      on: "ON",
      off: "OFF",
      diff: "diff",
      diffHint: "Auto-injected in diff review mode",
      placeholder: "Type here. Enter to send (Shift+Enter for newline)",
      ready: "ready",
      sending: "sending…",
      streaming: "streaming…",
      error: "error",
      copy: "Copy",
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
      send: "送信",
      stop: "停止",
      exportMd: "Markdown",
      clear: "消去",
      provider: "プロバイダ",
      model: "モデル",
      chatModel: "チャットモデル",
      codeModel: "コーディングモデル",
      baseUrl: "Base URL",
      apiKey: "APIキー",
      mode: "モード",
      persona: "ペルソナ",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "ストリーミング",
      on: "ON",
      off: "OFF",
      diff: "diff",
      diffHint: "diff批評モードで自動注入されます",
      placeholder: "ここに入力。Enterで送信 (Shift+Enterで改行)",
      ready: "ready",
      sending: "sending…",
      streaming: "streaming…",
      error: "error",
      copy: "コピー",
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
      send: "Envoyer",
      stop: "Stop",
      exportMd: "Markdown",
      clear: "Effacer",
      provider: "Fournisseur",
      model: "Modèle",
      chatModel: "Modèle (chat)",
      codeModel: "Modèle (code)",
      baseUrl: "Base URL",
      apiKey: "Clé API",
      mode: "Mode",
      persona: "Persona",
      temperature: "temperature",
      maxTokens: "max_tokens",
      timeoutSeconds: "timeout_seconds",
      stream: "Streaming",
      on: "ON",
      off: "OFF",
      diff: "diff",
      diffHint: "Auto-injecté en mode revue de diff",
      placeholder: "Tapez ici. Entrée pour envoyer (Maj+Entrée pour nouvelle ligne)",
      ready: "prêt",
      sending: "envoi…",
      streaming: "stream…",
      error: "erreur",
      copy: "Copier",
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
    { k: "openai-compatible", l: { ja: "OpenAI互換", en: "OpenAI compat", fr: "Compat OpenAI" } },
    { k: "anthropic", l: { ja: "Anthropic", en: "Anthropic", fr: "Anthropic" } },
  ];

  const MODES = [
    { k: "VIBE", l: { ja: "VIBE", en: "VIBE", fr: "VIBE" } },
    { k: "壁打ち", l: { ja: "壁打ち", en: "Ideation", fr: "Idéation" } },
    { k: "実況", l: { ja: "実況", en: "Live", fr: "Direct" } },
    { k: "diff批評", l: { ja: "diff批評", en: "Diff review", fr: "Revue diff" } },
  ];

  const PERSONAS = ["default", "novelist", "cynical", "cheerful", "thoughtful"];

  const PRESETS = {
    vibe: {
      vibe: true,
      provider: "mistral",
      chatModel: "devstral-2",
      codeModel: "devstral-2",
      baseUrl: "https://api.mistral.ai/v1",
      mode: "VIBE",
    },
    openai: {
      vibe: false,
      provider: "openai-compatible",
      chatModel: "gpt-4o-mini",
      codeModel: "gpt-4o-mini",
      baseUrl: "https://api.openai.com/v1",
      mode: "壁打ち",
    },
    anthropic: {
      vibe: false,
      provider: "anthropic",
      chatModel: "claude-3-5-sonnet-latest",
      codeModel: "claude-3-5-sonnet-latest",
      baseUrl: "https://api.anthropic.com/v1",
      mode: "壁打ち",
    },
  };

  const DEFAULT_CONFIG = {
    ...PRESETS.vibe,
    persona: "default",
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
      chat_model: strOrUndef(cfg.chatModel),
      code_model: strOrUndef(cfg.codeModel),
      base_url: strOrUndef(cfg.baseUrl),
      api_key: strOrUndef(apiKey),
      mode: strOrUndef(cfg.mode),
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
      lines.push(`## ${m.role}`, "", String(m.content || "").trimEnd(), "");
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
    "openai-compatible": "#60a5fa",
    "anthropic":         "#fb7185",
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
      const merged = { ...DEFAULT_CONFIG, ...v };
      // Backward-compat: older config used `model` only.
      if (!merged.chatModel && merged.model) merged.chatModel = merged.model;
      if (!merged.codeModel && merged.model) merged.codeModel = merged.model;
      if (!merged.codeModel && merged.chatModel) merged.codeModel = merged.chatModel;
      return merged;
    });
    const [apiKey, setApiKey] = useState("");
    const [diff, setDiff] = useState("");

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
          messages: Array.isArray(t.messages) ? t.messages : [],
        }));
      if (!threads.length) threads = [makeThread("Thread 1")];
      const active0 = (localStorage.getItem(LS.active) || "").trim();
      const active = threads.some((t) => t.id === active0) ? active0 : threads[0].id;
      return { threads, activeId: active };
    });

    const [input, setInput] = useState("");
    const [sending, setSending] = useState(false);

    const abortRef = useRef(null);
    const saveTimer = useRef(null);
    const chatBodyRef = useRef(null);

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
      fetch("/api/status")
        .then((r) => r.json())
        .then((j) => setStatus(j))
        .catch(() => {});
    }, []);

    const keyOpenAI =
      status && status.providers && status.providers["openai-compatible"]
        ? !!status.providers["openai-compatible"].api_key_present
        : false;
    const keyMistral = status && status.providers && status.providers.mistral ? !!status.providers.mistral.api_key_present : false;
    const keyAnthropic =
      status && status.providers && status.providers.anthropic ? !!status.providers.anthropic.api_key_present : false;

    const KeyDot = ({ ok }) => e("span", { className: "kdot " + (ok ? "ok" : "missing") });

    const providerLabel = (k) => {
      const p = PROVIDERS.find((x) => x.k === k);
      return (p && p.l && p.l[lang]) || k;
    };

    const modeLabel = (k) => {
      const m = MODES.find((x) => x.k === k);
      return (m && m.l && m.l[lang]) || k;
    };

    const usesCodeModel =
      config.mode === "VIBE" || config.mode === "diff批評" || config.mode === "ログ解析";
    const activeModel = usesCodeModel
      ? (config.codeModel || config.chatModel || "")
      : (config.chatModel || config.codeModel || "");

    const scrollBottom = () => {
      const el = chatBodyRef.current;
      if (!el) return;
      el.scrollTop = el.scrollHeight;
    };

    const applyPreset = (kind) => {
      const p = PRESETS[kind];
      if (!p) return;
      setConfig({ ...DEFAULT_CONFIG, ...p });
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
        chat_model: config.chatModel,
        code_model: config.codeModel,
        base_url: config.baseUrl,
        mode: config.mode,
        persona: config.persona,
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

    const appendDelta = (threadId, msgId, delta) => {
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
      requestAnimationFrame(scrollBottom);
    };

    const setMsg = (threadId, msgId, content) => {
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
      requestAnimationFrame(scrollBottom);
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

    const stop = () => {
      if (abortRef.current) abortRef.current.abort();
      abortRef.current = null;
      setSending(false);
    };

    const onSend = async () => {
      if (sending) return;
      const text = (input || "").trim();
      if (!text) return;

      const threadId = activeThread.id;
      const history = (activeThread.messages || []).map((m) => ({ role: m.role, content: m.content }));

      const userMsg = { id: uid(), role: "user", content: text, ts: Date.now() };
      const asstMsg = { id: uid(), role: "assistant", content: "", ts: Date.now(), streaming: true };

      setThreadState((s) => ({
        ...s,
        threads: s.threads.map((t) =>
          t.id === threadId ? { ...t, updatedAt: Date.now(), messages: [...(t.messages || []), userMsg, asstMsg] } : t
        ),
      }));
      ensureTitle(threadId, text);
      setInput("");
      setSending(true);
      requestAnimationFrame(scrollBottom);

      const reqBody = buildReq(config, apiKey, history, text, diff);
      const ac = new AbortController();
      abortRef.current = ac;

      try {
        if (config.stream) {
          await streamChat(
            reqBody,
            (evt) => {
              if (!evt) return;
              if (evt.event === "delta") {
                const j = safeJsonParse(evt.data || "{}", {});
                if (j && j.delta) appendDelta(threadId, asstMsg.id, j.delta);
              } else if (evt.event === "error") {
                const j = safeJsonParse(evt.data || "{}", {});
                throw new Error(j.error || tr(lang, "error"));
              }
            },
            ac.signal
          );
        } else {
          const j = await postJson("/api/chat", reqBody, ac.signal);
          setMsg(threadId, asstMsg.id, String((j && j.content) || ""));
        }
      } catch (err) {
        const msg = (err && err.message) || String(err || "error");
        if (config.stream && !ac.signal.aborted) {
          try {
            const j = await postJson("/api/chat", reqBody, ac.signal);
            setMsg(threadId, asstMsg.id, String((j && j.content) || ""));
          } catch (err2) {
            const msg2 = (err2 && err2.message) || String(err2 || "error");
            setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg2}`);
          }
        } else if (ac.signal.aborted) {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "stop")}]`);
        } else {
          setMsg(threadId, asstMsg.id, `[${tr(lang, "error")}] ${msg}`);
        }
      } finally {
        setSending(false);
        abortRef.current = null;
      }
    };

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
              e(KeyDot, { ok: keyMistral }),
              " M  ",
              e(KeyDot, { ok: keyOpenAI }),
              " O  ",
              e(KeyDot, { ok: keyAnthropic }),
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
            )
          )
        )
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
                  e("div", { className: "preset-sub" }, "Mistral · devstral-2")
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
                      onChange: (ev) => setConfig({ ...config, provider: ev.target.value }),
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
                    value: config.chatModel,
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
                  value: config.codeModel,
                  onChange: (ev) => setConfig({ ...config, codeModel: ev.target.value }),
                })
              ),
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
                    MODES.map((m) => e("option", { key: m.k, value: m.k }, (m.l && m.l[lang]) || m.k))
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
          { className: "panel chat" },
          e(
            "div",
            { className: "panel-header" },
            e("h2", null, tr(lang, "chat") + ": " + (activeThread ? activeThread.title : "")),
            e("div", { style: { display: "flex", gap: 8, alignItems: "center" } },
              e("span", {
                style: {
                  width: 8, height: 8, borderRadius: "50%", display: "inline-block",
                  background: PROVIDER_COLORS[config.provider] || "rgba(255,255,255,0.4)",
                  boxShadow: "0 0 8px " + (PROVIDER_COLORS[config.provider] || "transparent") + "88",
                  transition: "background 400ms ease, box-shadow 400ms ease",
                },
              }),
              e("span", { className: "pill" }, providerLabel(config.provider) + " · " + modeLabel(config.mode) + (activeModel ? " · " + activeModel : "")),
            )
          ),
          e("div", { className: "chat-body", ref: chatBodyRef }, (activeThread.messages || []).map(renderMessage)),
          e(
            "div",
            { className: "composer" },
            e("textarea", {
              className: "textarea",
              value: input,
              placeholder: tr(lang, "placeholder"),
              onChange: (ev) => setInput(ev.target.value),
              onKeyDown: (ev) => {
                if ((ev.metaKey || ev.ctrlKey) && ev.key === "Enter") {
                  ev.preventDefault();
                  onSend();
                  return;
                }
                if (ev.key === "Enter" && !ev.shiftKey) {
                  ev.preventDefault();
                  onSend();
                }
              },
            }),
            sending
              ? e("button", { className: "btn btn-warn", onClick: stop }, tr(lang, "stop"))
              : e("button", { className: "btn btn-primary", onClick: onSend }, tr(lang, "send"))
          ),
          e(
            "div",
            { className: "statusline" },
            e("span", {
              className: "dot",
              style: {
                background: sending
                  ? (PROVIDER_COLORS[config.provider] || "rgba(45,212,191,0.85)")
                  : "rgba(45,212,191,0.85)",
                transition: "background 400ms ease",
              },
            }),
            e("span", null, sending ? (config.stream ? tr(lang, "streaming") : tr(lang, "sending")) : tr(lang, "ready")),
            activeThread && activeThread.messages && activeThread.messages.length > 0 && e("span", {
              style: { marginLeft: "auto", color: "var(--faint)", fontSize: 11, fontFamily: "var(--mono)" },
            }, "~" + activeThread.messages.reduce((s, m) => s + estimateTokens(m.content), 0) + " tokens · " + activeThread.messages.length + " msgs"),
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
