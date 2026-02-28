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

  // SECTION: app (filled below)
  function App() {
    return e(
      "div",
      { className: "app" },
      e(
        "div",
        { className: "topbar" },
        e(
          "div",
          { className: "topbar-inner" },
          e("div", { className: "brand" }, e("h1", null, "OBSTRAL"), e("span", { className: "pill" }, "UI"))
        )
      ),
      e(
        "div",
        { className: "main" },
        e("div", { className: "panel" }, e("div", { className: "panel-body" }, "Loading…"))
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
