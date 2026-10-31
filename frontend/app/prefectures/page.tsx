import type { Metadata } from "next";
import { socialMetadata } from "../../lib/social-metadata";
import PrefectureDirectory from "../../components/prefecture-directory";

export const metadata: Metadata = socialMetadata(
  "Préfectures — Qomune",
  "Les 101 préfectures départementales et leurs profils de démonstration sur Qomune, prototype indépendant.",
  "/prefectures",
);

export default function PrefecturesPage() {
  return <PrefectureDirectory />;
}
