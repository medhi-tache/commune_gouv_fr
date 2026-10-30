import type { Metadata } from "next";
import { socialMetadata } from "../lib/social-metadata";
import "./globals.css";

export const metadata: Metadata = {
  ...socialMetadata(
    "Qomune — Votre commune, en lien direct",
    "Découvrez Qomune, le prototype indépendant de publication des communes : actualités locales et espace des élus.",
    "/",
  ),
  robots: { index: false, follow: false },
};
export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="fr">
      <body>{children}</body>
    </html>
  );
}
