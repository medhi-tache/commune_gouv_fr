import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Qomune — Votre commune, en lien direct",
  description:
    "Prototype de publication des communes : actualités publiques et espace du conseil municipal.",
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
