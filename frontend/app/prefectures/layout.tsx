import Dashboard from "../../components/dashboard";
import "../gouvernement/government.css";

export default function PrefecturesLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <Dashboard section="prefectures">{children}</Dashboard>;
}
