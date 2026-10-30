import { socialMetadata } from "../../lib/social-metadata";
import Administration from "../../components/administration";
import "./administration.css";

export const metadata = {
  ...socialMetadata(
    "Administration de démonstration — Qomune",
    "Explorez les graphiques fictifs de la démonstration publique du prototype Qomune.",
    "/administration",
  ),
  robots: { index: false, follow: false },
};
export default function Page() {
  return <Administration demo />;
}
