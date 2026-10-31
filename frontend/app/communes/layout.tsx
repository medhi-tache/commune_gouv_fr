import Dashboard from "../../components/dashboard";
import "../gouvernement/government.css";
export default function CommunesLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <Dashboard section="communes">{children}</Dashboard>;
}
