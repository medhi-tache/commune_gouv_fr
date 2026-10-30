import {
  NextResponse,
  type NextFetchEvent,
  type NextRequest,
} from "next/server";

/** Une navigation HTML = une page vue. Aucun cookie, IP, email ou paramètre n'est transmis. */
export function proxy(request: NextRequest, event: NextFetchEvent) {
  const key = process.env.QOMUNE_METRICS_KEY;
  const prefetch =
    request.headers.has("next-router-prefetch") ||
    request.headers.get("purpose") === "prefetch" ||
    request.headers.get("sec-purpose")?.includes("prefetch");
  if (
    key &&
    request.method === "GET" &&
    request.headers.get("accept")?.includes("text/html") &&
    !request.headers.has("rsc") &&
    !prefetch
  ) {
    const page =
      request.nextUrl.pathname === "/" ||
      request.nextUrl.pathname.startsWith("/gouvernement") ||
      request.nextUrl.pathname.startsWith("/presidence")
        ? "public"
        : request.nextUrl.pathname === "/administration"
          ? "admin_demo"
          : "admin_private";
    event.waitUntil(
      fetch(
        `${process.env.API_BASE_URL || "http://127.0.0.1:8080"}/internal/visits`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Qomune-Metrics-Key": key,
          },
          body: JSON.stringify({ page }),
          signal: AbortSignal.timeout(3000),
          cache: "no-store",
        },
      )
        .then((response) => {
          if (!response.ok) console.error("Mesure des visites indisponible.");
        })
        .catch(() => {
          console.error("Mesure des visites indisponible.");
        }),
    );
  }
  return NextResponse.next();
}
export const config = {
  matcher: [
    "/",
    "/gouvernement/:path*",
    "/presidence/:path*",
    "/administration",
    "/administration/connexion",
  ],
};
