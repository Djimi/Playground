import { existsSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";

const ROOT = process.cwd();
const FRONTEND_URL = process.env.FRONTEND_URL ?? "http://127.0.0.1:5173";
const FRONTEND_PORT = process.env.FRONTEND_PORT ?? (new URL(FRONTEND_URL).port || "5173");
const STARTUP_TIMEOUT_MS = Number(process.env.STARTUP_TIMEOUT_MS ?? 60_000);
const POLL_INTERVAL_MS = 500;

let composeStarted = false;
let frontend;
let frontendExit;
let frontendResult;
let shuttingDown = false;

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: ROOT,
    encoding: "utf8",
    ...options,
  });

  if (result.error) throw result.error;
  return result;
}

function preflight() {
  const majorNodeVersion = Number(process.versions.node.split(".")[0]);
  if (majorNodeVersion < 22) {
    throw new Error(`Node.js 22 or newer is required; found ${process.versions.node}`);
  }

  if (!existsSync("compose.yaml")) {
    throw new Error("compose.yaml was not found; run this command from the repository root");
  }

  if (!existsSync("node_modules/vite/bin/vite.js")) {
    throw new Error("Vite is not installed; run npm install first");
  }

  let docker;
  try {
    docker = run("docker", ["compose", "version"], { stdio: "pipe" });
  } catch (error) {
    throw new Error(
      `Docker Compose is required; install Docker Desktop or the Docker Compose plugin (${error.message})`,
    );
  }
  if (docker.status !== 0) {
    throw new Error("Docker Compose is required; install Docker Desktop or the Docker Compose plugin");
  }

  const config = run("docker", ["compose", "config", "--quiet"], { stdio: "pipe" });
  if (config.status !== 0) {
    throw new Error(`Docker Compose configuration is invalid\n${config.stderr.trim()}`.trim());
  }

  if (!Number.isFinite(STARTUP_TIMEOUT_MS) || STARTUP_TIMEOUT_MS <= 0) {
    throw new Error("STARTUP_TIMEOUT_MS must be a positive number");
  }
}

function startCompose() {
  console.log("Starting PostGIS and GraphQL API...");
  composeStarted = true;
  const result = run("docker", ["compose", "up", "-d", "db", "api"], {
    stdio: "inherit",
  });
  if (result.status !== 0) {
    throw new Error("Docker Compose could not start db and api");
  }
}

function apiUrl() {
  const result = run("docker", ["compose", "port", "api", "3000"], { stdio: "pipe" });
  if (result.status !== 0) {
    throw new Error("Docker Compose did not publish the API port");
  }

  const port = result.stdout.trim().match(/:(\d+)$/)?.[1];
  if (!port) {
    throw new Error(`Could not determine the published API port from: ${result.stdout.trim()}`);
  }
  return `http://127.0.0.1:${port}`;
}

async function checkApi(url) {
  const response = await fetch(`${url}/graphql`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ query: "{ playgrounds(limit: 1) { id } }" }),
    signal: AbortSignal.timeout(3_000),
  });
  const body = await response.json();

  if (!response.ok) throw new Error(`GraphQL returned HTTP ${response.status}`);
  if (body.errors?.length) throw new Error(body.errors.map(({ message }) => message).join("; "));
  if (!Array.isArray(body.data?.playgrounds)) {
    throw new Error("GraphQL response did not contain playground data");
  }
}

async function checkFrontend() {
  const response = await fetch(FRONTEND_URL, { signal: AbortSignal.timeout(3_000) });
  if (!response.ok) throw new Error(`Frontend returned HTTP ${response.status}`);
}

async function waitFor(label, check) {
  const deadline = Date.now() + STARTUP_TIMEOUT_MS;
  let lastError;

  while (Date.now() < deadline) {
    try {
      await check();
      return;
    } catch (error) {
      if (error?.fatal) throw error;
      lastError = error;
      await sleep(POLL_INTERVAL_MS);
    }
  }

  const detail = lastError instanceof Error ? `: ${lastError.message}` : "";
  throw new Error(`${label} did not become ready within ${STARTUP_TIMEOUT_MS}ms${detail}`);
}

function startFrontend() {
  console.log("Starting Vite frontend...");
  frontend = spawn(
    process.execPath,
    [
      "node_modules/vite/bin/vite.js",
      "--host",
      "127.0.0.1",
      "--port",
      FRONTEND_PORT,
      "--strictPort",
    ],
    { cwd: ROOT, env: { ...process.env, BROWSER: "none" }, stdio: "inherit" },
  );
  frontendExit = new Promise((resolve) => {
    frontend.once("error", (error) => {
      frontendResult = { error };
      resolve(frontendResult);
    });
    frontend.once("exit", (code, signal) => {
      frontendResult = { code, signal };
      resolve(frontendResult);
    });
  });
}

async function stopFrontend() {
  if (!frontend || frontend.exitCode !== null) return;

  shuttingDown = true;
  frontend.kill("SIGINT");
  const result = await Promise.race([frontendExit, sleep(3_000).then(() => null)]);
  if (!result && frontend.exitCode === null) frontend.kill("SIGTERM");
}

function printComposeDiagnostics() {
  if (!composeStarted) return;
  console.error("\nRecent Docker Compose logs:");
  run("docker", ["compose", "logs", "--tail=80", "db", "api"], { stdio: "inherit" });
}

async function main() {
  preflight();
  startCompose();

  const url = apiUrl();
  await waitFor("GraphQL API", () => checkApi(url));

  startFrontend();
  await waitFor("Frontend", async () => {
    if (frontendResult) {
      const error = frontendResult.error ?? new Error(`Vite exited before readiness (code=${frontendResult.code ?? "signal"})`);
      error.fatal = true;
      throw error;
    }
    await checkFrontend();
  });

  console.log(`\nLocal application is ready.`);
  console.log(`Frontend: ${FRONTEND_URL}`);
  console.log(`GraphQL:  ${url}/graphql`);
  console.log("Press Ctrl+C to stop the frontend. Docker services remain available for reuse.");

  const result = await frontendExit;
  if (!shuttingDown) {
    if (result.error) throw result.error;
    throw new Error(`Vite exited unexpectedly (code=${result.code ?? "signal"})`);
  }
}

process.on("SIGINT", () => {
  process.exitCode = 130;
  void stopFrontend();
});
process.on("SIGTERM", () => {
  process.exitCode = 143;
  void stopFrontend();
});

try {
  await main();
} catch (error) {
  await stopFrontend();
  console.error(`\nLocal startup failed: ${error instanceof Error ? error.message : error}`);
  printComposeDiagnostics();
  process.exitCode = 1;
}
