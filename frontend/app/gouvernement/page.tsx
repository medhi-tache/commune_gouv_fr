import type { Metadata } from "next";
import { socialMetadata } from "../../lib/social-metadata";
import GovernmentDirectory from "../../components/government-directory";

export const metadata: Metadata = socialMetadata(
  "Gouvernement — Qomune",
  "Les membres du Gouvernement et leurs fonctions dans le prototype indépendant Qomune. Consultez leurs fiches et les profils officiels.",
  "/gouvernement",
);

export default function GovernmentPage() {
  return <GovernmentDirectory />;
}
