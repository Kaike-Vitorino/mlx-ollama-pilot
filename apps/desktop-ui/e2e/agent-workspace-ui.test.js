import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { JSDOM } from "jsdom";

const indexHtml = await readFile(new URL("../ui/index.html", import.meta.url), "utf8");
const appJs = await readFile(new URL("../ui/app.js", import.meta.url), "utf8");

function jsonResponse(data, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    async text() {
      return data == null ? "" : JSON.stringify(data);
    },
    async json() {
      return data;
    },
  };
}

async function flush(count = 4) {
  for (let index = 0; index < count; index += 1) {
    await new Promise((resolve) => setImmediate(resolve));
  }
}

function createFixture({
  modelsResponse,
  cachedModels,
  cachedCurrentModel,
  openClawAvailable = true,
} = {}) {
  let sessions = [
    { id: "sess-1", name: "Operacao", message_count: 3 },
    { id: "sess-2", name: "Channels", message_count: 1 },
  ];
  let nextSession = 3;
  const fetchCalls = [];

  const dom = new JSDOM(indexHtml, {
    url: "http://localhost/",
    runScripts: "outside-only",
    pretendToBeVisual: true,
  });

  const { window } = dom;
  const { document } = window;

  window.__MLX_PILOT_DAEMON_URL__ = "http://127.0.0.1:11436";
  window.localStorage.setItem("mlxPilotDaemonUrl", "http://127.0.0.1:11435");
  if (cachedModels) {
    window.localStorage.setItem("mlxPilotModelCache", JSON.stringify(cachedModels));
  }
  if (cachedCurrentModel) {
    window.localStorage.setItem("mlxPilotCurrentModel", cachedCurrentModel);
  }

  Object.defineProperty(window.HTMLCanvasElement.prototype, "getContext", {
    configurable: true,
    value() {
      return {
        beginPath() {},
        moveTo() {},
        lineTo() {},
        stroke() {},
        arc() {},
        fill() {},
        clearRect() {},
      };
    },
  });

  window.requestAnimationFrame = () => 1;
  window.cancelAnimationFrame = () => {};
  window.setTimeout = (callback) => {
    callback();
    return 1;
  };
  window.clearTimeout = () => {};
  window.alert = () => {};
  window.confirm = () => true;
  window.prompt = () => "slack";
  window.open = () => {};

  Object.defineProperty(window.navigator, "clipboard", {
    configurable: true,
    value: {
      async writeText() {},
    },
  });

  window.fetch = async (url, options = {}) => {
    const requestUrl = new URL(url, "http://localhost/");
    const path = `${requestUrl.pathname}${requestUrl.search}`;
    const method = options.method || "GET";
    const body = options.body ? JSON.parse(options.body) : null;

    fetchCalls.push({ method, path, body, url: requestUrl.toString() });

    if (path === "/health") {
      return jsonResponse({ status: "ok", provider: "ollama" });
    }

    if (path === "/config") {
      return jsonResponse({
        active_agent_framework: "openclaw",
        models_dir: "G:/models",
        openclaw_cli_path: "G:/bin/openclaw.exe",
        openclaw_state_dir: "G:/state/openclaw",
      });
    }

    if (path === "/models") {
      return jsonResponse(modelsResponse ?? [
        {
          id: "mlx-community/Qwen3-4B-4bit",
          name: "Qwen3 4B",
          provider: "ollama",
          is_available: true,
        },
      ]);
    }

    if (path === "/openclaw/status") {
      return jsonResponse({
        available: openClawAvailable,
        cli_exists: openClawAvailable,
      });
    }

    if (path === "/agent/config" && method === "GET") {
      return jsonResponse({
        provider: "ollama",
        model_id: "mlx-community/Qwen3-4B-4bit",
        execution_mode: "full",
        approval_mode: "ask",
      });
    }

    if (path === "/agent/config" && method === "POST") {
      return jsonResponse(body);
    }

    if (path === "/agent/sessions" && method === "GET") {
      return jsonResponse(sessions);
    }

    if (path === "/agent/sessions" && method === "POST") {
      const created = {
        id: `sess-${nextSession}`,
        name: body?.name || `Sessao ${nextSession}`,
        message_count: 0,
      };
      nextSession += 1;
      sessions = [created, ...sessions];
      return jsonResponse(created);
    }

    if (path === "/agent/plugins") {
      return jsonResponse([
        { id: "memory", enabled: true, description: "Persistencia local" },
        { id: "auth", enabled: false, description: "Broker de identidade" },
      ]);
    }

    if (path === "/agent/plugins/enable" || path === "/agent/plugins/disable") {
      return jsonResponse({});
    }

    if (path === "/agent/skills/check") {
      return jsonResponse({
        skills: [
          { name: "planner", active: true },
          { name: "channels", enabled: true },
          { name: "browser", active: false },
        ],
      });
    }

    if (path === "/agent/skills/enable" || path === "/agent/skills/disable") {
      return jsonResponse({});
    }

    if (path === "/agent/tools") {
      return jsonResponse([
        { name: "read_file", enabled: true },
        { name: "list_dir", enabled: true },
        { name: "exec", enabled: false },
      ]);
    }

    if (path === "/agent/channels") {
      return jsonResponse([
        { channel_id: "whatsapp", accounts: [{ account_id: "ops", status: "connected" }] },
        { channel_id: "slack", accounts: [] },
      ]);
    }

    if (path === "/agent/channels/upsert" || path === "/agent/channels/remove") {
      return jsonResponse({});
    }

    if (path === "/agent/audit?limit=30") {
      return jsonResponse({
        entries: [
          {
            event_type: "tool_call",
            tool_name: "read_file",
            summary: "Resumo de auditoria",
            timestamp: "2026-04-15T12:00:00Z",
          },
        ],
      });
    }

    if (path === "/environment?reveal=false" || path === "/environment?reveal=true") {
      return jsonResponse({ variables: [] });
    }

    if (path === "/agent/run" && method === "POST") {
      return jsonResponse({
        session_id: body?.session_id || "sess-1",
        final_response: "Resposta do agent",
        total_tokens: 128,
        latency_ms: 900,
      });
    }

    throw new Error(`Unhandled request: ${method} ${path}`);
  };

  window.eval(appJs);

  return {
    window,
    document,
    fetchCalls,
    cleanup() {
      dom.window.close();
    },
  };
}

test("agent workspace boots with live summary and toggles config tab", async () => {
  const fixture = createFixture();

  try {
    await flush();

    assert.ok(fixture.fetchCalls.some((entry) => entry.url.startsWith("http://127.0.0.1:11436/")));
    assert.equal(fixture.document.getElementById("agent-daemon-status")?.textContent, "Online");
    assert.equal(fixture.document.getElementById("agent-session-count")?.textContent, "2");
    assert.equal(fixture.document.getElementById("agent-plugin-count")?.textContent, "1");
    assert.equal(fixture.document.getElementById("agent-skill-count")?.textContent, "2");
    assert.equal(fixture.document.getElementById("agent-channel-count")?.textContent, "2");
    assert.equal(fixture.document.getElementById("agent-audit-count")?.textContent, "1");

    fixture.document.querySelector('.tab[data-panel="agent"]')?.click();
    fixture.document.querySelector('.agent-view-tab[data-agent-view="config"]')?.click();
    await flush(2);

    assert.ok(fixture.document.getElementById("agent-view-config")?.classList.contains("active"));
    assert.equal(fixture.document.getElementById("agent-view-config")?.style.display, "block");
  } finally {
    fixture.cleanup();
  }
});

test("agent workspace prompts, submits runs, and creates sessions", async () => {
  const fixture = createFixture();

  try {
    await flush();

    fixture.document.querySelector(".agent-prompt-card")?.click();
    const input = fixture.document.getElementById("agent-command-input");
    assert.match(input?.value || "", /Revise a configuracao atual do agent/i);

    fixture.document.getElementById("agent-send-btn")?.click();
    await flush();

    const runCall = fixture.fetchCalls.find((entry) => entry.path === "/agent/run");
    assert.ok(runCall);
    assert.equal(runCall.body.model_id, "mlx-community/Qwen3-4B-4bit");
    assert.match(fixture.document.getElementById("agent-chat-messages")?.textContent || "", /Resposta do agent/);

    fixture.document.getElementById("btn-new-session")?.click();
    await flush();

    assert.equal(fixture.document.getElementById("agent-session-count")?.textContent, "3");
    assert.equal(fixture.document.getElementById("btn-export-session")?.disabled, false);
  } finally {
    fixture.cleanup();
  }
});

test("workspace hides OpenClaw when unavailable and preserves cached model shell", async () => {
  const fixture = createFixture({
    modelsResponse: [],
    cachedModels: [
      {
        id: "cached/qwen-local",
        name: "Qwen Local",
        provider: "mlx",
        is_available: true,
      },
    ],
    cachedCurrentModel: "cached/qwen-local",
    openClawAvailable: false,
  });

  try {
    await flush();

    assert.ok(fixture.document.getElementById("tab-openclaw")?.classList.contains("hidden"));
    assert.equal(fixture.document.getElementById("current-model")?.textContent, "Qwen Local");
    assert.match(fixture.document.getElementById("installed-count")?.textContent || "", /modelo/);
  } finally {
    fixture.cleanup();
  }
});
