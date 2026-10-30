/** Lance les deux serveurs et les arrête ensemble, y compris si l'un échoue. */
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { randomBytes } from "node:crypto";

const root = fileURLToPath(new URL("../", import.meta.url));
// Un secret propre à cette installation locale ; jamais de mot de passe administrateur public.
const adminPasswordFile = resolve(
  root,
  "backend/data/administration-password.txt",
);
if (!process.env.QOMUNE_ADMIN_PASSWORD) {
  if (!existsSync(adminPasswordFile)) {
    mkdirSync(resolve(root, "backend/data"), { recursive: true });
    writeFileSync(adminPasswordFile, randomBytes(24).toString("base64url"), {
      mode: 0o600,
      flag: "wx",
    });
  }
  process.env.QOMUNE_ADMIN_PASSWORD = readFileSync(
    adminPasswordFile,
    "utf8",
  ).trim();
  console.log(
    `Mot de passe administrateur local : fichier ${adminPasswordFile}`,
  );
}
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

function launch(command, args, directory, env = {}, onOutput) {
  const child = spawn(command, args, {
    cwd: resolve(root, directory),
    stdio: onOutput ? ["inherit", "pipe", "inherit"] : "inherit",
    env: { ...process.env, ...env },
  });
  children.push(child);
  if (onOutput)
    child.stdout.on("data", (chunk) => {
      process.stdout.write(chunk);
      onOutput(chunk.toString());
    });
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
  "Communes Insee ; comptes et publications de démonstration. Ctrl+C arrête les deux services.\n",
);
// Le premier import peut prendre quelques secondes. L'interface ne démarre qu'après la disponibilité de l'API.
let backendOutput = "";
let webStarted = false;
launch(
  "cargo",
  ["run", "--locked"],
  "backend",
  { API_BIND: apiBind },
  (output) => {
    backendOutput = (backendOutput + output).slice(-2000);
    if (!webStarted && !stopping && backendOutput.includes("API Qomune :")) {
      webStarted = true;
      launch(
        process.platform === "win32" ? "npm.cmd" : "npm",
        ["run", "dev", "--", "--port", webPort],
        "frontend",
        {
          API_BASE_URL: process.env.API_BASE_URL || `http://${apiBind}`,
        },
      );
    }
  },
);
process.on("SIGINT", () => stop());
process.on("SIGTERM", () => stop());
