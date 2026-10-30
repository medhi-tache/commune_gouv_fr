"use client";

import { useEffect, useState, type FormEvent } from "react";
import { LockKeyhole, LogOut, ShieldCheck } from "lucide-react";
import GovernmentNationalPublisher from "./government-national-publisher";
import { api } from "../lib/api";
import type { GovernmentMember } from "../lib/government";

type DemoAccount = {
  slug: string;
  name: string;
  function: string;
  email: string;
  is_demo: boolean;
};

/** Une session de démonstration distincte, sans droit sur les espaces municipaux. */
export default function GovernmentDemoAccount({
  member,
}: {
  member: GovernmentMember;
}) {
  const email = `${member.slug}@gouvernement.demo.qomune.fr`;
  const [account, setAccount] = useState<DemoAccount | null>(null);
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    let current = true;
    api<{ account: DemoAccount | null }>("government-demo/me")
      .then((result) => {
        if (current) setAccount(result.account);
      })
      .catch((error: Error) => {
        if (current) setError(error.message);
      })
      .finally(() => {
        if (current) setLoading(false);
      });
    return () => {
      current = false;
    };
  }, []);

  async function login(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      await api("government-demo/login", { email, password });
      const result = await api<{ account: DemoAccount }>("government-demo/me");
      setAccount(result.account);
      setPassword("");
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function logout() {
    setBusy(true);
    setError("");
    try {
      await api("government-demo/logout", {});
      setAccount(null);
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section
      className="government-demo"
      aria-labelledby="government-demo-title"
    >
      <span className="government-demo-badge">
        DÉMONSTRATION · COMPTE FICTIF
      </span>
      <h2 id="government-demo-title">
        <LockKeyhole size={20} /> Tester ce compte
      </h2>
      <p>
        Ce compte de test est inspiré de la fiche publique de {member.name}. Il
        n’appartient pas à cette personne et ne représente aucune communication
        officielle.
      </p>
      {loading ? (
        <p role="status">Chargement de la session…</p>
      ) : account ? (
        <div className="government-demo-session">
          <h3>
            <ShieldCheck size={19} /> Session de démonstration ouverte
          </h3>
          <p>
            <strong>{account.name}</strong> · compte fictif
          </p>
          <p>{account.function}</p>
          <p className="government-demo-email">{account.email}</p>
          <p>
            Cette session permet de diffuser des publications nationales de
            démonstration. Elle ne donne aucun accès aux conseils municipaux ni
            à l’administration.
          </p>
          {account.slug !== member.slug && (
            <p>
              Pour tester le compte de {member.name}, déconnectez d’abord cette
              session.
            </p>
          )}
          {account.slug === member.slug && (
            <GovernmentNationalPublisher
              key={account.slug}
              name={account.name}
            />
          )}
          <button className="button" disabled={busy} onClick={logout}>
            <LogOut size={16} /> Se déconnecter du compte fictif
          </button>
        </div>
      ) : (
        <form className="government-demo-form" onSubmit={login}>
          <p>
            Identifiants publics, réservés au prototype. Mot de passe :{" "}
            <strong>gouvernement-demo</strong>
          </p>
          <label>
            Identifiant de démonstration
            <input
              type="email"
              value={email}
              readOnly
              autoComplete="username"
            />
          </label>
          <label>
            Mot de passe de démonstration
            <input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              autoComplete="current-password"
              maxLength={200}
              required
            />
          </label>
          <button className="button" disabled={busy}>
            <LockKeyhole size={16} />{" "}
            {busy ? "Connexion…" : "Ouvrir la session de démonstration"}
          </button>
        </form>
      )}
      {error && (
        <p className="government-demo-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
