/** Relais interne vers Rust. Les cookies restent HttpOnly et les réponses ne sont jamais mises en cache. */
import { NextRequest } from "next/server";

export const dynamic = "force-dynamic";
const apiBase = process.env.API_BASE_URL || "http://127.0.0.1:8080";

async function proxy(
  request: NextRequest,
  context: { params: Promise<{ path: string[] }> },
) {
  const { path } = await context.params;
  const allowed =
    /^(communes(?:\/meta)?|health|members|auth\/(login|logout|me)|publications(?:\/[a-zA-Z0-9-]+)?)$/;
  const route = path.join("/");
  if (!allowed.test(route))
    return Response.json({ error: "Route introuvable." }, { status: 404 });

  // Une page tierce ne peut pas déclencher une connexion ou une publication à notre place.
  if (request.method !== "GET") {
    const origin = request.headers.get("origin");
    // Next.js peut reconstruire son URL avec « localhost ». Host conserve l'adresse réellement visitée.
    // Derrière un proxy HTTPS, APP_ORIGIN définit explicitement l'origine publique autorisée.
    const expectedUrl = new URL(request.url);
    expectedUrl.host = request.headers.get("host") || expectedUrl.host;
    const expectedOrigin = process.env.APP_ORIGIN || expectedUrl.origin;
    if (!origin || origin !== expectedOrigin) {
      return Response.json(
        { error: "Origine de la requête refusée." },
        { status: 403 },
      );
    }
    if (!request.headers.get("content-type")?.startsWith("application/json")) {
      return Response.json(
        { error: "Un contenu JSON est attendu." },
        { status: 415 },
      );
    }
  }
  try {
    const body = request.method === "GET" ? undefined : await request.text();
    if (body && new TextEncoder().encode(body).length > 65536)
      return Response.json(
        { error: "Contenu trop volumineux." },
        { status: 413 },
      );
    const response = await fetch(
      `${apiBase}/${route}${request.nextUrl.search}`,
      {
        method: request.method,
        headers: {
          "Content-Type": "application/json",
          cookie: request.headers.get("cookie") || "",
        },
        body,
        cache: "no-store",
        signal: AbortSignal.timeout(15000),
      },
    );
    const headers = new Headers({
      "Content-Type": "application/json",
      "Cache-Control": "no-store, private",
    });
    const cookie = response.headers.get("set-cookie");
    if (cookie) headers.set("set-cookie", cookie);
    const text = await response.text();
    // Les rejets de structure d'Axum sont parfois en texte : l'interface reçoit toujours du JSON français.
    let payload: unknown;
    try {
      payload = JSON.parse(text);
    } catch {
      payload = {
        error:
          "Le contenu de la requête est invalide. Vérifiez les champs transmis.",
      };
    }
    return new Response(JSON.stringify(payload), {
      status: response.status,
      headers,
    });
  } catch {
    return Response.json(
      {
        error:
          "Le serveur est indisponible. Vérifiez que le backend Rust est démarré.",
      },
      { status: 503 },
    );
  }
}
export { proxy as GET, proxy as POST };
