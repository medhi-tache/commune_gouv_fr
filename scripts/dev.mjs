/** Lance les deux serveurs et les arrête ensemble, y compris si l'un échoue. */
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

const root = fileURLToPath(new URL("../", import.meta.url));
const webPort = process.env.WEB_PORT || "3000";
const apiBind = process.env.API_BIND || "127.0.0.1:8080";
const children = [];
let stopping = false;

function stop(code = 0) {
  if (stopping) return;
  stopping = true;
  for (const child of children) child.kill("SIGTERM");
  // Les processus disposent d'un délai pour libérer leurs ports avant un arrêt forcé.
  const timer = setTimeout(() => {
    for (const child of children) child.kill("SIGKILL");
    process.exit(code);
  }, 3000);
  Promise.all(
    children.map(
      (child) =>
        new Promise((done) => {
          if (child.exitCode !== null || child.signalCode !== null) done();
          else child.once("exit", done);
        }),
    ),
  ).then(() => {
    clearTimeout(timer);
    process.exit(code);
  });
}

function launch(command, args, directory, env = {}) {
  const child = spawn(command, args, {
    cwd: resolve(root, directory),
    stdio: "inherit",
    env: { ...process.env, ...env },
  });
  children.push(child);
  child.once("error", (error) => {
    console.error(`Impossible de lancer ${command} : ${error.message}`);
    stop(1);
  });
  child.once("exit", (code) => {
    if (!stopping) stop(code || 1);
  });
}

console.log(`\nQomune — prototype local : http://127.0.0.1:${webPort}`);
console.log(
  "Les données et les comptes sont fictifs. Ctrl+C arrête les deux services.\n",
);
launch("cargo", ["run", "--locked"], "backend", { API_BIND: apiBind });
launch(
  process.platform === "win32" ? "npm.cmd" : "npm",
  ["run", "dev", "--", "--port", webPort],
  "frontend",
  {
    API_BASE_URL: process.env.API_BASE_URL || `http://${apiBind}`,
  },
);
process.on("SIGINT", () => stop());
process.on("SIGTERM", () => stop());
