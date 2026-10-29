"use client";

import { useEffect, useState, type FormEvent } from "react";
import {
  Check,
  Copy,
  LockKeyhole,
  Plus,
  ShieldCheck,
  Users,
} from "lucide-react";
import { api } from "../lib/api";

type CouncilMember = {
  id: string;
  name: string;
  email: string;
  role: string;
  commune_id: string;
};
type CreatedMember = { member: CouncilMember; initial_password: string };

/** Gestion réservée au maire. La commune et le rôle du nouveau membre sont imposés par Rust. */
export default function CouncilMembers({
  commune,
  onBusyChange,
}: {
  commune: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const [members, setMembers] = useState<CouncilMember[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [created, setCreated] = useState<CreatedMember | null>(null);
  const [copied, setCopied] = useState(false);
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    let current = true;
    setLoading(true);
    setLoadError("");
    api<CouncilMember[]>("members")
      .then((items) => {
        if (current) setMembers(items);
      })
      .catch((error) => {
        if (current) setLoadError(error.message);
      })
      .finally(() => {
        if (current) setLoading(false);
      });
    return () => {
      current = false;
    };
  }, [revision]);

  async function addMember(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    setBusy(true);
    onBusyChange(true);
    setError("");
    setCopied(false);
    try {
      const result = await api<CreatedMember>("members", {
        name: data.get("name"),
        email: data.get("email"),
      });
      setCreated(result);
      setRevision((value) => value + 1);
      form.reset();
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setBusy(false);
      onBusyChange(false);
    }
  }

  async function copyCredentials() {
    if (!created) return;
    try {
      await navigator.clipboard.writeText(
        `Connexion Qomune\nAdresse e-mail : ${created.member.email}\nMot de passe initial : ${created.initial_password}`,
      );
      setCopied(true);
    } catch {
      setError(
        "La copie automatique est indisponible. Sélectionnez les identifiants pour les copier manuellement.",
      );
    }
  }

  return (
    <div className="council-management">
      <p className="dialog-intro">
        Le conseil municipal{" "}
        {(/^[aeiouyhàâéèêëîïôùûü]/i.test(commune) ? "d’" : "de ") + commune}. En
        tant que maire, vous pouvez ajouter les conseillers de votre commune.
      </p>
      <div className="member-section-title">
        <h3>
          <Users size={17} /> Membres actuels
        </h3>
        <span className="count-badge">{loading ? "…" : members.length}</span>
      </div>
      {loading ? (
        <p className="privacy-note" role="status">
          Chargement des membres…
        </p>
      ) : loadError ? (
        <div className="form-error" role="alert">
          {loadError}{" "}
          <button
            className="text-button"
            onClick={() => setRevision((value) => value + 1)}
          >
            Réessayer
          </button>
        </div>
      ) : (
        <ul className="member-list">
          {members.map((person) => (
            <li key={person.id}>
              <span className="avatar">
                {person.name
                  .split(" ")
                  .map((word) => word[0])
                  .slice(0, 2)
                  .join("")}
              </span>
              <div className="member-identity">
                <strong>{person.name}</strong>
                <span>{person.email}</span>
              </div>
              <span
                className={`member-role ${person.role === "maire" ? "is-mayor" : ""}`}
              >
                {person.role === "maire" ? "Maire" : "Conseiller municipal"}
              </span>
            </li>
          ))}
        </ul>
      )}

      {created && (
        <section className="member-created" aria-labelledby="created-title">
          <h3 id="created-title">
            <Check size={18} /> Le compte de {created.member.name} est créé
          </h3>
          <p>
            Notez ces identifiants avant de fermer cette fenêtre. Le mot de
            passe initial ne sera plus consultable.
          </p>
          <label>
            Adresse e-mail
            <input
              readOnly
              value={created.member.email}
              onFocus={(event) => event.target.select()}
            />
          </label>
          <label>
            Mot de passe initial
            <input
              readOnly
              value={created.initial_password}
              onFocus={(event) => event.target.select()}
            />
          </label>
          <p>
            Aucun e-mail n’a été envoyé. Transmettez ces identifiants au membre
            par un moyen privé ; il pourra se connecter avec son adresse e-mail.
          </p>
          <div className="form-actions">
            <button className="button" onClick={copyCredentials}>
              {copied ? <Check size={15} /> : <Copy size={15} />}{" "}
              {copied ? "Identifiants copiés" : "Copier les identifiants"}
            </button>
            <button
              className="button primary"
              onClick={() => {
                setCreated(null);
                setError("");
              }}
            >
              Ajouter un autre membre
            </button>
          </div>
        </section>
      )}

      {!created && (
        <form className="form member-form" onSubmit={addMember}>
          <h3>
            <Plus size={17} /> Ajouter un membre
          </h3>
          <label>
            Nom complet
            <input
              name="name"
              required
              minLength={2}
              maxLength={120}
              autoComplete="off"
              placeholder="Prénom et nom du conseiller"
              disabled={busy}
            />
          </label>
          <label>
            Adresse e-mail du membre
            <input
              name="email"
              type="email"
              required
              maxLength={200}
              autoComplete="off"
              placeholder="prenom.nom@exemple.fr"
              disabled={busy}
            />
          </label>
          <div className="demo-notice">
            <ShieldCheck size={20} />
            <p>
              Le compte sera rattaché à {commune}, avec le rôle de conseiller
              municipal et l’accès aux publications privées de ce conseil. Un
              mot de passe initial sera généré à la création.
            </p>
          </div>
          <button className="button primary full" disabled={busy}>
            <Plus size={16} />
            {busy ? "Création du compte…" : "Créer le compte du membre"}
          </button>
        </form>
      )}
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      <p className="privacy-note">
        <LockKeyhole size={13} /> Cette gestion est réservée au maire de la
        commune.
      </p>
    </div>
  );
}
