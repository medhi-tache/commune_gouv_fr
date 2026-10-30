"use client";

import { useEffect, useRef, useState, type FormEvent } from "react";
import { Globe2 } from "lucide-react";
import { api } from "../lib/api";

type NationalPost = {
  id: string;
  title: string;
  body: string;
  published_at: string;
};

/** Un identifiant conservé lors d'un nouvel essai évite de diffuser deux fois le même formulaire. */
export default function GovernmentNationalPublisher({
  name,
}: {
  name: string;
}) {
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
    setBusy(true);
    setError("");
    setSuccess("");
    requestId.current ||= crypto.randomUUID();
    try {
      await api("government-demo/publications", {
        title,
        body,
        category,
        request_id: requestId.current,
      });
      setTitle("");
      setBody("");
      requestId.current = null;
      setSuccess(
        "Votre publication de démonstration est visible dans le fil public de toutes les communes.",
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
        <Globe2 size={21} /> Publier dans toutes les communes
      </h3>
      <p>
        Diffusion publique nationale, en une seule action, depuis le compte
        fictif de <strong>{name}</strong>.
      </p>
      <p className="government-national-notice">
        Chaque message porte automatiquement la mention « Démonstration » et
        précise qu’il ne constitue pas une communication officielle.
      </p>
      <form className="government-demo-form" onSubmit={publish}>
        <label>
          Titre de la publication nationale
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
          Catégorie de la publication nationale
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
          Message de la publication nationale
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
          Destinataires : toutes les communes. Visibilité : publique. Signature
          : {name}, compte fictif.
        </p>
        <button className="button" disabled={busy}>
          <Globe2 size={17} />{" "}
          {busy ? "Diffusion en cours…" : "Publier dans toutes les communes"}
        </button>
      </form>
      {success && (
        <p className="government-national-success" role="status">
          {success} <a href="/">Voir le fil public</a>
        </p>
      )}
      {error && (
        <p className="government-demo-error" role="alert">
          {error}
        </p>
      )}
      <div className="government-national-history">
        <h3>Vos dernières diffusions nationales</h3>
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
                  · Toutes les communes
                </p>
              </li>
            ))}
          </ul>
        ) : (
          <p>Aucune diffusion nationale pour ce compte.</p>
        )}
        {posts.length === 20 && (
          <p>Les 20 diffusions les plus récentes sont affichées.</p>
        )}
      </div>
    </section>
  );
}
