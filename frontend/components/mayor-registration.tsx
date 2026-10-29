"use client";

import { useState, type FormEvent } from "react";
import { ArrowRight, Info } from "lucide-react";
import CommunePicker, { type Commune } from "./commune-picker";
import { api } from "../lib/api";

/** Inscription déclarative du prototype : aucune vérification d'identité ou de mandat n'est simulée. */
export default function MayorRegistration({
  initialCommune,
  onBusyChange,
  onRegistered,
}: {
  initialCommune: Commune | null;
  onBusyChange: (busy: boolean) => void;
  onRegistered: () => Promise<void>;
}) {
  const [commune, setCommune] = useState(initialCommune);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  async function register(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError("");
    if (!commune) {
      setError("Choisissez votre commune.");
      return;
    }
    const data = new FormData(event.currentTarget);
    setBusy(true);
    onBusyChange(true);
    try {
      await api("auth/register", {
        name: data.get("name"),
        email: data.get("email"),
        password: data.get("password"),
        commune_id: commune.id,
      });
      await onRegistered();
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setBusy(false);
      onBusyChange(false);
    }
  }
  return (
    <>
      <div className="demo-notice">
        <Info size={20} />
        <div>
          <strong>Inscription non vérifiée — prototype</strong>
          <p>
            Votre identité et votre mandat de maire ne sont pas vérifiés dans
            cette démonstration. FranceConnect et le contrôle du mandat via le
            RNE sont prévus pour la production.
          </p>
        </div>
      </div>
      <form className="form" onSubmit={register}>
        <label>
          Nom complet
          <input
            name="name"
            autoComplete="name"
            required
            minLength={2}
            maxLength={120}
            disabled={busy}
          />
        </label>
        <div className="registration-commune">
          <span>Votre commune</span>
          <CommunePicker
            label="Choisir la commune du maire"
            allowAll={false}
            value={commune}
            onChange={setCommune}
            disabled={busy}
          />
          {commune && (
            <small>
              {commune.department} · INSEE {commune.id}
            </small>
          )}
        </div>
        <label>
          Adresse e-mail
          <input
            type="email"
            name="email"
            autoComplete="username"
            required
            maxLength={200}
            disabled={busy}
          />
        </label>
        <label>
          Mot de passe
          <input
            type="password"
            name="password"
            autoComplete="new-password"
            required
            minLength={12}
            maxLength={200}
            disabled={busy}
          />
          <small>Au moins 12 caractères.</small>
        </label>
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        <button className="button primary full" disabled={busy}>
          {busy ? "Création du compte…" : "Créer mon compte de maire"}
          <ArrowRight size={17} />
        </button>
      </form>
    </>
  );
}
