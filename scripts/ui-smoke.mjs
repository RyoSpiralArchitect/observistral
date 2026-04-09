import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");

function parseArgs(argv) {
  const parsed = {
    workspaceRoot: repoRoot,
    host: "127.0.0.1",
    port: 18992,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--workspace-root" && argv[i + 1]) {
      parsed.workspaceRoot = path.resolve(argv[++i]);
    } else if (arg === "--host" && argv[i + 1]) {
      parsed.host = argv[++i];
    } else if (arg === "--port" && argv[i + 1]) {
      parsed.port = Number(argv[++i]) || parsed.port;
    }
  }
  return parsed;
}

function nowStamp() {
  return String(Date.now());
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForServer(url, timeoutMs = 20000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch (_) {
      // retry
    }
    await sleep(250);
  }
  throw new Error(`timed out waiting for ${url}`);
}

function includesAny(text, candidates) {
  return candidates.some((candidate) => text.includes(candidate));
}

async function captureViewport(page, url, name, viewport, outDir) {
  await page.setViewportSize(viewport);
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 30000 });
  await page.waitForSelector(".panel", { timeout: 15000 });
  await page.waitForSelector("h2", { timeout: 15000 });
  await page.waitForTimeout(1200);

  const bodyText = (await page.locator("body").innerText()).replace(/\s+/g, " ");
  const headings = await page.locator("h2").allTextContents();
  const horizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth + 4
  );

  const screenshotPath = path.join(outDir, `${name}.png`);
  await page.screenshot({ path: screenshotPath, fullPage: true });

  const checks = {
    harnessReviews: includesAny(bodyText, [
      "Harness reviews",
      "ハーネスレビュー",
      "Revues du harnais",
    ]),
    runtimeApprovals: includesAny(bodyText, [
      "Runtime approvals",
      "ランタイム承認",
      "Approbations runtime",
    ]),
    settings: includesAny(bodyText, [
      "Settings",
      "設定",
      "Réglages",
    ]),
    horizontalOverflow: !horizontalOverflow,
  };

  return {
    name,
    viewport,
    headings,
    screenshotPath,
    checks,
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const outDir = path.join(repoRoot, ".tmp", `ui_smoke_${nowStamp()}`);
  await mkdir(outDir, { recursive: true });

  const serverArgs = [
    "run",
    "--quiet",
    "--manifest-path",
    path.join(repoRoot, "Cargo.toml"),
    "--",
    "serve",
    "--host",
    args.host,
    "--port",
    String(args.port),
  ];

  const server = spawn("cargo", serverArgs, {
    cwd: args.workspaceRoot,
    stdio: ["ignore", "pipe", "pipe"],
  });

  let serverStdout = "";
  let serverStderr = "";
  server.stdout.on("data", (chunk) => {
    serverStdout += chunk.toString();
  });
  server.stderr.on("data", (chunk) => {
    serverStderr += chunk.toString();
  });

  const pageErrors = [];
  const consoleErrors = [];
  const badResponses = [];
  let browser;
  const reportPath = path.join(outDir, "report.json");
  try {
    const baseUrl = `http://${args.host}:${args.port}/`;
    await waitForServer(baseUrl);

    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    page.on("pageerror", (err) => {
      pageErrors.push(String(err.message || err));
    });
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });
    page.on("response", (response) => {
      if (response.status() >= 400) {
        badResponses.push({
          url: response.url(),
          status: response.status(),
        });
      }
    });

    const viewports = [
      { name: "desktop", viewport: { width: 1440, height: 1600 } },
      { name: "mobile", viewport: { width: 430, height: 1400 } },
    ];
    const results = [];
    for (const entry of viewports) {
      results.push(
        await captureViewport(page, baseUrl, entry.name, entry.viewport, outDir)
      );
    }

    const report = {
      ok:
        pageErrors.length === 0
        && badResponses.length === 0
        && results.every((result) =>
          Object.values(result.checks).every(Boolean)
        ),
      workspaceRoot: args.workspaceRoot,
      url: baseUrl,
      pageErrors,
      consoleErrors,
      badResponses,
      results,
    };
    await writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`, "utf8");
    console.log(`[ui-smoke] report=${reportPath}`);
    for (const result of results) {
      console.log(
        `[ui-smoke] ${result.name} checks=${JSON.stringify(result.checks)} screenshot=${result.screenshotPath}`
      );
    }
    if (!report.ok) {
      process.exitCode = 1;
    }
  } catch (error) {
    const failureReport = {
      ok: false,
      workspaceRoot: args.workspaceRoot,
      pageErrors,
      consoleErrors,
      badResponses,
      error: String(error && error.stack ? error.stack : error),
    };
    await writeFile(reportPath, `${JSON.stringify(failureReport, null, 2)}\n`, "utf8");
    throw error;
  } finally {
    if (browser) {
      await browser.close();
    }
    server.kill("SIGINT");
    await new Promise((resolve) => {
      server.once("exit", resolve);
      setTimeout(resolve, 5000);
    });
    if (serverStderr.trim()) {
      await writeFile(path.join(outDir, "server.stderr.log"), serverStderr, "utf8");
    }
    if (serverStdout.trim()) {
      await writeFile(path.join(outDir, "server.stdout.log"), serverStdout, "utf8");
    }
  }
}
try {
  await main();
} catch (error) {
  console.error(error);
  process.exitCode = 1;
}
