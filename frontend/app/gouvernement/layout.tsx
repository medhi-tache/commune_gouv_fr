import Dashboard from "../../components/dashboard";
import "./government.css";

export default function GovernmentLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <Dashboard>{children}</Dashboard>;
}
