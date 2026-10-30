import { socialMetadata } from "../../../lib/social-metadata";
import Administration from "../../../components/administration";
import "../administration.css";

export const metadata = {
  ...socialMetadata(
    "Administration privée — Qomune",
    "Connexion à l’administration protégée du prototype Qomune.",
    "/administration/connexion",
  ),
  robots: { index: false, follow: false },
};
export default function Page() {
  return <Administration />;
}
