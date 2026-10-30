"use client";

import { useEffect, useRef, useState, type FormEvent } from "react";
import { Globe2 } from "lucide-react";
import CommunePicker, { type Commune } from "./commune-picker";
import { api } from "../lib/api";

type NationalPost = {
  id: string;
  title: string;
  body: string;
  published_at: string;
  scope: string;
  commune_id: string;
  commune_name?: string;
};

/** Un identifiant conservé lors d'un nouvel essai évite de diffuser deux fois le même formulaire. */
export default function GovernmentNationalPublisher({
  name,
}: {
  name: string;
}) {
  const [audience, setAudience] = useState<"national" | "commune">("national");
  const [commune, setCommune] = useState<Commune | null>(null);
  const [publishedLink, setPublishedLink] = useState("/");
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [category, setCategory] = useState("Services publics");
  const [posts, setPosts] = useState<NationalPost[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const requestId = useRef<string | null>(null);

  useEffect(() => {
    let current = true;
    api<NationalPost[]>("government-demo/publications")
      .then((items) => {
        if (current) setPosts(items);
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

  function edit() {
    requestId.current = null;
    setSuccess("");
  }
  async function publish(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (audience === "commune" && !commune) {
      setError("Choisissez une commune destinataire.");
      return;
    }
    setBusy(true);
    setError("");
    setSuccess("");
    requestId.current ||= crypto.randomUUID();
    try {
      const receipt = await api<{
        scope: string;
        commune_id: string;
        commune_name?: string;
      }>("government-demo/publications", {
        title,
        body,
        category,
        request_id: requestId.current,
        ...(audience === "commune" ? { commune_id: commune!.id } : {}),
      });
      setTitle("");
      setBody("");
      requestId.current = null;
      setSuccess(
        receipt.scope === "national"
          ? "Votre publication de démonstration est visible dans le fil public de toutes les communes."
          : `Votre publication de démonstration est visible dans le fil public de la commune : ${receipt.commune_name || receipt.commune_id}.`,
      );
      setPublishedLink(
        receipt.scope === "national"
          ? "/"
          : `/?commune=${encodeURIComponent(receipt.commune_id)}`,
      );
      // Une erreur de relecture ne transforme pas une publication réussie en invitation à republier.
      try {
        setPosts(await api<NationalPost[]>("government-demo/publications"));
      } catch {
        setError(
          "La publication est enregistrée, mais l’historique n’a pas pu être actualisé. Rechargez la page.",
        );
      }
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section
      className="government-national"
      aria-labelledby="government-national-title"
    >
      <h3 id="government-national-title">
        <Globe2 size={21} /> Publier un message de démonstration
      </h3>
      <p>
        Diffusion publique vers toutes les communes ou une commune choisie,
        depuis le compte fictif de <strong>{name}</strong>.
      </p>
      <p className="government-national-notice">
        Chaque message porte automatiquement la mention « Démonstration » et
        précise qu’il ne constitue pas une communication officielle.
      </p>
      <form className="government-demo-form" onSubmit={publish}>
        <fieldset className="government-audience" disabled={busy}>
          <legend>Destinataires de la publication</legend>
          <label>
            <input
              type="radio"
              name="government-audience"
              value="national"
              checked={audience === "national"}
              onChange={() => {
                edit();
                setAudience("national");
              }}
            />{" "}
            Toutes les communes
          </label>
          <label>
            <input
              type="radio"
              name="government-audience"
              value="commune"
              checked={audience === "commune"}
              onChange={() => {
                edit();
                setAudience("commune");
              }}
            />{" "}
            Une commune en particulier
          </label>
        </fieldset>
        {audience === "commune" && (
          <CommunePicker
            label="Commune destinataire"
            value={commune}
            onChange={(selected) => {
              edit();
              setCommune(selected);
            }}
            allowAll={false}
            disabled={busy}
          />
        )}
        <label>
          Titre de la publication
          <input
            value={title}
            onChange={(event) => {
              edit();
              setTitle(event.target.value);
            }}
            minLength={3}
            maxLength={140}
            required
            disabled={busy}
          />
        </label>
        <label>
          Catégorie de la publication
          <select
            value={category}
            onChange={(event) => {
              edit();
              setCategory(event.target.value);
            }}
            disabled={busy}
          >
            {[
              "Services publics",
              "Vie citoyenne",
              "Travaux",
              "Culture",
              "Environnement",
            ].map((item) => (
              <option key={item}>{item}</option>
            ))}
          </select>
        </label>
        <label>
          Message de la publication
          <textarea
            value={body}
            onChange={(event) => {
              edit();
              setBody(event.target.value);
            }}
            rows={7}
            minLength={10}
            maxLength={10000}
            required
            disabled={busy}
          />
        </label>
        <p>
          Destinataires :{" "}
          {audience === "national"
            ? "toutes les communes"
            : commune?.name || "choisissez une commune"}
          . Visibilité : publique. Signature : {name}, compte fictif.
        </p>
        <button
          className="button"
          disabled={busy || (audience === "commune" && !commune)}
        >
          <Globe2 size={17} />{" "}
          {busy
            ? "Diffusion en cours…"
            : audience === "national"
              ? "Publier dans toutes les communes"
              : `Publier pour ${commune?.name || "la commune choisie"}`}
        </button>
      </form>
      {success && (
        <p className="government-national-success" role="status">
          {success} <a href={publishedLink}>Voir le fil public</a>
        </p>
      )}
      {error && (
        <p className="government-demo-error" role="alert">
          {error}
        </p>
      )}
      <div className="government-national-history">
        <h3>Vos dernières publications</h3>
        {loading ? (
          <p role="status">Chargement des diffusions…</p>
        ) : posts.length ? (
          <ul>
            {posts.map((post) => (
              <li key={post.id}>
                <strong>{post.title}</strong>
                <p>
                  <time dateTime={post.published_at}>
                    {new Date(post.published_at).toLocaleString("fr-FR", {
                      timeZone: "Europe/Paris",
                    })}
                  </time>{" "}
                  ·{" "}
                  {post.scope === "national"
                    ? "Toutes les communes"
                    : post.commune_name || post.commune_id}
                </p>
              </li>
            ))}
          </ul>
        ) : (
          <p>Aucune publication pour ce compte.</p>
        )}
        {posts.length === 20 && (
          <p>Les 20 diffusions les plus récentes sont affichées.</p>
        )}
      </div>
    </section>
  );
}
