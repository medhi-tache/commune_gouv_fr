import Administration from "../../components/administration";
import "./administration.css";

export const metadata = {
  title: "Administration — Qomune",
  robots: { index: false, follow: false },
};
export default function Page() {
  return <Administration demo />;
}
