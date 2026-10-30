/** Vérifie le filtrage des navigations réelles sans faire de requête réseau. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import vm from "node:vm";
import { AsyncLocalStorage } from "node:async_hooks";
// Next.js attend cette primitive globale dans son environnement de test serveur.
globalThis.AsyncLocalStorage = AsyncLocalStorage;
const require = createRequire(
  new URL("../frontend/package.json", import.meta.url),
);
const ts = require("typescript");
const { NextRequest } = require("next/server");
const {
  unstable_doesMiddlewareMatch,
} = require("next/experimental/testing/server");
const source = readFileSync(
  new URL("../frontend/proxy.ts", import.meta.url),
  "utf8",
);
const code = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS },
}).outputText;
const calls = [],
  errors = [],
  environment = {
    QOMUNE_METRICS_KEY: "cle-privee-de-test-avec-au-moins-32-caracteres",
    API_BASE_URL: "http://api:8080",
  };
let fail = false;
const context = {
  exports: {},
  require,
  process: { env: environment },
  AbortSignal,
  console: { error: (value) => errors.push(value) },
  fetch: async (...args) => {
    calls.push(args);
    if (fail) throw new Error("Erreur réseau simulée");
    return { ok: true };
  },
};
vm.runInNewContext(code, context);
const { proxy, config } = context.exports;
for (const path of [
  "/",
  "/gouvernement",
  "/gouvernement/sebastien-lecornu",
  "/administration",
  "/administration/connexion",
]) {
  assert.equal(
    unstable_doesMiddlewareMatch({ config, nextConfig: {}, url: path }),
    true,
  );
}
for (const path of [
  "/api/health",
  "/api/admin/statistics",
  "/favicon.ico",
  "/documents/cahier_des_charges_qomune.pdf",
  "/_next/static/chunk.js",
]) {
  assert.equal(
    unstable_doesMiddlewareMatch({ config, nextConfig: {}, url: path }),
    false,
  );
}
async function visit(path, headers = {}, method = "GET") {
  calls.length = 0;
  const pending = [];
  const response = proxy(
    new NextRequest("https://qomune.fr" + path, { method, headers }),
    { waitUntil: (promise) => pending.push(promise) },
  );
  assert.equal(response.headers.get("x-middleware-next"), "1");
  assert.equal(response.headers.get("set-cookie"), null);
  await Promise.all(pending);
}
for (const [path, page] of [
  ["/?email=ne-pas-transmettre@example.fr", "public"],
  ["/gouvernement", "public"],
  ["/gouvernement/sebastien-lecornu", "public"],
  ["/administration", "admin_demo"],
  ["/administration/connexion", "admin_private"],
]) {
  await visit(path, {
    accept: "text/html",
    cookie: "session=privee",
    "x-forwarded-for": "192.0.2.1",
  });
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], "http://api:8080/internal/visits");
  assert.deepEqual(JSON.parse(calls[0][1].body), { page });
  assert.equal(Object.keys(calls[0][1].headers).length, 2);
}
for (const headers of [
  {},
  { accept: "*/*" },
  { accept: "text/html", rsc: "1" },
  { accept: "text/html", "next-router-prefetch": "1" },
  { accept: "text/html", purpose: "prefetch" },
  { accept: "text/html", "sec-purpose": "prefetch;prerender" },
]) {
  await visit("/", headers);
  assert.equal(calls.length, 0);
}
await visit("/", { accept: "text/html" }, "POST");
assert.equal(calls.length, 0);
fail = true;
await visit("/", { accept: "text/html" });
assert.equal(errors.length, 1);
delete environment.QOMUNE_METRICS_KEY;
await visit("/", { accept: "text/html" });
assert.equal(calls.length, 0);
console.log(
  "Filtrage des pages vues vérifié : navigations, préchargements, contrôles de santé, absence de cookies et indisponibilité de la mesure.",
);
