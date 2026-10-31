"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Globe2, MapPin, Newspaper } from "lucide-react";
import { api } from "../lib/api";

type Publication = {
  id: string;
  title: string;
  body: string;
  category: string;
  published_at: string;
  scope: string;
  commune_id: string;
  commune_name?: string;
  author_name: string;
  author_role: string;
  department_name?: string;
};

type PublicationPage = {
  publications: Publication[];
  next_cursor: string | null;
};

/** Les visiteurs consultent les messages publics du seul auteur du profil affiché. */
export default function GovernmentPublications({
  slug,
  name,
  institution = "government",
}: {
  slug: string;
  name: string;
  institution?: "government" | "prefectures";
}) {
  const [posts, setPosts] = useState<Publication[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const generation = useRef(0);
  const pending = useRef(false);

  const load = useCallback(
    async (before?: string) => {
      if (pending.current) return;
      pending.current = true;
      const current = generation.current;
      setLoading(true);
      setError("");
      try {
        const page = await api<PublicationPage>(
          `${institution}/${encodeURIComponent(slug)}/publications${before ? `?before=${encodeURIComponent(before)}` : ""}`,
        );
        if (current !== generation.current) return;
        setPosts((previous) => {
          if (!before) return page.publications;
          const ids = new Set(previous.map((post) => post.id));
          return [
            ...previous,
            ...page.publications.filter((post) => !ids.has(post.id)),
          ];
        });
        setCursor(page.next_cursor);
      } catch (error) {
        if (current === generation.current) setError((error as Error).message);
      } finally {
        if (current === generation.current) {
          pending.current = false;
          setLoading(false);
        }
      }
    },
    [slug, institution],
  );

  useEffect(() => {
    generation.current += 1;
    pending.current = false;
    setPosts([]);
    setCursor(null);
    void load();
    // Une nouvelle publication est relue sans attendre un rechargement du profil.
    const refresh = () => {
      // La relecture après publication remplace une pagination éventuellement en cours.
      generation.current += 1;
      pending.current = false;
      void load();
    };
    window.addEventListener("qomune:government-publication", refresh);
    return () => {
      generation.current += 1;
      window.removeEventListener("qomune:government-publication", refresh);
    };
  }, [load]);

  return (
    <section
      className="government-profile-publications"
      aria-labelledby="profile-publications-title"
      aria-busy={loading}
    >
      <h2 id="profile-publications-title">
        <Newspaper size={22} /> Publications de {name}
      </h2>
      <p className="government-note">
        Messages publics du compte fictif de démonstration. Ils ne constituent
        pas des communications officielles.
      </p>
      {!loading && !error && posts.length === 0 && (
        <p>Aucune publication publique pour ce profil pour le moment.</p>
      )}
      <div className="government-profile-posts">
        {posts.map((post) => (
          <article className="government-profile-post" key={post.id}>
            <div className="government-profile-post-meta">
              <span className="government-demo-badge">DÉMONSTRATION</span>
              <span>{post.category}</span>
              <time dateTime={post.published_at}>
                {new Date(post.published_at).toLocaleString("fr-FR", {
                  timeZone: "Europe/Paris",
                })}
              </time>
            </div>
            <h3>{post.title}</h3>
            <p className="government-profile-post-body">{post.body}</p>
            <p className="government-profile-post-author">
              {post.author_name} · {post.author_role}
            </p>
            {post.scope === "department" ? (
              <p className="government-profile-post-audience">
                <MapPin size={16} /> Toutes les communes du département :{" "}
                {post.department_name}
              </p>
            ) : (
              <a
                className="government-profile-post-audience"
                href={
                  post.scope === "national"
                    ? "/"
                    : `/?commune=${encodeURIComponent(post.commune_id)}`
                }
              >
                {post.scope === "national" ? (
                  <Globe2 size={16} />
                ) : (
                  <MapPin size={16} />
                )}
                {post.scope === "national"
                  ? "Toutes les communes"
                  : post.commune_name || post.commune_id}{" "}
                · Voir le fil public
              </a>
            )}
          </article>
        ))}
      </div>
      {loading && <p role="status">Chargement des publications…</p>}
      {error && (
        <p className="government-demo-error" role="alert">
          {error}
        </p>
      )}
      {error ? (
        <button
          className="button"
          disabled={loading}
          onClick={() => void load(posts.length && cursor ? cursor : undefined)}
        >
          Réessayer
        </button>
      ) : (
        cursor && (
          <button
            className="button"
            disabled={loading}
            onClick={() => void load(cursor)}
          >
            Voir les publications précédentes
          </button>
        )
      )}
    </section>
  );
}
