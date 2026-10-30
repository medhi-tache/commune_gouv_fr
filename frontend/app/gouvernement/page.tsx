import type { Metadata } from "next";
import GovernmentDirectory from "../../components/government-directory";

export const metadata: Metadata = {
  title: "Gouvernement — Qomune",
  description:
    "Les membres du Gouvernement, leurs fonctions et leurs profils officiels.",
};

export default function GovernmentPage() {
  return <GovernmentDirectory />;
}
