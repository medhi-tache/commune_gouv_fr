"use client";

import { useEffect, useRef, useState, type FormEvent } from "react";
import { LockKeyhole, MessageSquare, RefreshCw, Send } from "lucide-react";
import { api } from "../lib/api";

type ReactionKind = "like" | "love" | "haha" | "wow" | "sad" | "angry";
type Comment = {
  id: string;
  author_id: string;
  author_name: string;
  author_role: string;
  body: string;
  created_at: string;
};
type ReactionSummary = {
  total: number;
  mine: ReactionKind | null;
  counts: Record<ReactionKind, number>;
  members: { member_id: string; author_name: string; reaction: ReactionKind }[];
};
type Discussion = {
  comments: Comment[];
  comments_count: number;
  next_cursor: string | null;
  reactions: ReactionSummary;
};
const reactions: { kind: ReactionKind; emoji: string; label: string }[] = [
  { kind: "like", emoji: "👍", label: "J’aime" },
  { kind: "love", emoji: "❤️", label: "J’adore" },
  { kind: "haha", emoji: "😆", label: "Haha" },
  { kind: "wow", emoji: "😮", label: "Waouh" },
  { kind: "sad", emoji: "😢", label: "Triste" },
  { kind: "angry", emoji: "😡", label: "Grrr" },
];
const dateLabel = (date: string) =>
  new Intl.DateTimeFormat("fr-FR", {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(date));

/** Les échanges restent internes au conseil, même si le texte principal de la publication est public. */
export default function CouncilDiscussion({
  publicationId,
  focus,
}: {
  publicationId: string;
  focus?: "comment" | "reaction";
}) {
  const [data, setData] = useState<Discussion | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [body, setBody] = useState("");
  const [posting, setPosting] = useState(false);
  const [reacting, setReacting] = useState(false);
  const [olderLoading, setOlderLoading] = useState(false);
  const [revision, setRevision] = useState(0);
  const textarea = useRef<HTMLTextAreaElement>(null);
  const firstReaction = useRef<HTMLButtonElement>(null);
  const focused = useRef(false);

  useEffect(() => {
    let current = true;
    setLoading(true);
    setError("");
    api<Discussion>(`publications/${publicationId}/discussion`)
      .then((result) => {
        if (current) setData(result);
      })
      .catch((error) => {
        if (current) setError(error.message);
      })
      .finally(() => {
        if (current) setLoading(false);
      });
    return () => {
      current = false;
    };
  }, [publicationId, revision]);
  useEffect(() => {
    if (loading || !data || focused.current) return;
    if (focus === "comment") textarea.current?.focus();
    if (focus === "reaction") firstReaction.current?.focus();
    focused.current = true;
  }, [loading, data, focus]);

  async function comment(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!body.trim() || posting || !data) return;
    setPosting(true);
    setError("");
    try {
      const created = await api<Comment>(
        `publications/${publicationId}/comments`,
        { body },
      );
      setData((previous) =>
        previous
          ? {
              ...previous,
              comments: [...previous.comments, created],
              comments_count: previous.comments_count + 1,
            }
          : previous,
      );
      setBody("");
      textarea.current?.focus();
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setPosting(false);
    }
  }
  async function react(kind: ReactionKind) {
    if (reacting || !data) return;
    setReacting(true);
    setError("");
    try {
      const result = await api<ReactionSummary>(
        `publications/${publicationId}/reaction`,
        { reaction: data.reactions.mine === kind ? null : kind },
      );
      setData((previous) =>
        previous ? { ...previous, reactions: result } : previous,
      );
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setReacting(false);
    }
  }
  async function older() {
    if (!data?.next_cursor || olderLoading) return;
    setOlderLoading(true);
    setError("");
    try {
      const result = await api<Discussion>(
        `publications/${publicationId}/discussion?before=${encodeURIComponent(data.next_cursor)}`,
      );
      setData((previous) =>
        previous
          ? {
              ...previous,
              comments: [...result.comments, ...previous.comments].filter(
                (comment, index, items) =>
                  items.findIndex((item) => item.id === comment.id) === index,
              ),
              next_cursor: result.next_cursor,
              comments_count: result.comments_count,
            }
          : previous,
      );
    } catch (error) {
      setError((error as Error).message);
    } finally {
      setOlderLoading(false);
    }
  }

  return (
    <section
      className="council-discussion"
      aria-label="Échanges du conseil municipal"
    >
      <div className="discussion-heading">
        <h3>
          <MessageSquare size={18} /> Échanges du conseil
        </h3>
        <button
          type="button"
          className="icon-button"
          aria-label="Actualiser les échanges"
          disabled={loading || posting || reacting || olderLoading}
          onClick={() => setRevision((value) => value + 1)}
        >
          <RefreshCw size={15} />
        </button>
      </div>
      <p className="discussion-privacy">
        <LockKeyhole size={13} /> Commentaires et réactions réservés aux membres
        de votre conseil, même sur une publication publique.
      </p>
      {loading ? (
        <p className="discussion-status" role="status">
          Chargement des échanges…
        </p>
      ) : (
        data && (
          <>
            <div className="reaction-picker" aria-label="Choisir une réaction">
              {reactions.map((reaction, index) => (
                <button
                  type="button"
                  ref={index === 0 ? firstReaction : undefined}
                  key={reaction.kind}
                  className={
                    data.reactions.mine === reaction.kind ? "selected" : ""
                  }
                  aria-label={reaction.label}
                  aria-pressed={data.reactions.mine === reaction.kind}
                  disabled={reacting}
                  onClick={() => react(reaction.kind)}
                >
                  <span aria-hidden="true">{reaction.emoji}</span>
                  <span>{reaction.label}</span>
                  <small>{data.reactions.counts[reaction.kind]}</small>
                </button>
              ))}
            </div>
            <div className="discussion-summary">
              <span>
                {data.reactions.total} réaction
                {data.reactions.total === 1 ? "" : "s"}
              </span>
              <span>
                {data.comments_count} commentaire
                {data.comments_count === 1 ? "" : "s"}
              </span>
            </div>
            {data.reactions.mine && (
              <p className="reaction-help">
                Cliquez à nouveau sur votre réaction pour la retirer.
              </p>
            )}
            {data.reactions.members.length > 0 && (
              <details className="reaction-members">
                <summary>Voir les réactions des membres</summary>
                <ul>
                  {data.reactions.members.map((person) => (
                    <li key={person.member_id}>
                      <span aria-hidden="true">
                        {
                          reactions.find(
                            (reaction) => reaction.kind === person.reaction,
                          )?.emoji
                        }
                      </span>{" "}
                      {person.author_name} ·{" "}
                      {
                        reactions.find(
                          (reaction) => reaction.kind === person.reaction,
                        )?.label
                      }
                    </li>
                  ))}
                </ul>
              </details>
            )}
            {data.next_cursor && (
              <button
                type="button"
                className="text-button"
                disabled={olderLoading}
                onClick={older}
              >
                {olderLoading
                  ? "Chargement…"
                  : "Voir les commentaires précédents"}
              </button>
            )}
            <ol className="comment-list">
              {data.comments.map((comment) => (
                <li key={comment.id}>
                  <span className="avatar">
                    {comment.author_name
                      .split(" ")
                      .map((name) => name[0])
                      .slice(0, 2)
                      .join("")}
                  </span>
                  <div className="comment-content">
                    <div className="comment-author">
                      <strong>{comment.author_name}</strong>
                      <span>
                        {comment.author_role === "maire"
                          ? "Maire"
                          : "Conseiller municipal"}
                      </span>
                    </div>
                    <p>{comment.body}</p>
                    <time dateTime={comment.created_at}>
                      {dateLabel(comment.created_at)}
                    </time>
                  </div>
                </li>
              ))}
            </ol>
            {!data.comments_count && (
              <p className="discussion-status">
                Commencez la discussion avec votre conseil.
              </p>
            )}
            <form className="comment-form" onSubmit={comment}>
              <label htmlFor={`comment-${publicationId}`}>
                Votre commentaire
              </label>
              <textarea
                ref={textarea}
                id={`comment-${publicationId}`}
                value={body}
                onChange={(event) => setBody(event.target.value)}
                maxLength={2000}
                required
                rows={3}
                placeholder="Écrivez un commentaire pour votre conseil…"
                disabled={posting}
              />
              <div>
                <small>{body.length}/2 000 caractères</small>
                <button
                  className="button primary"
                  disabled={posting || !body.trim()}
                >
                  <Send size={14} />
                  {posting ? "Envoi…" : "Commenter"}
                </button>
              </div>
            </form>
          </>
        )
      )}
      {error && (
        <div className="form-error" role="alert">
          {error}
          {!data && (
            <button
              type="button"
              className="text-button"
              onClick={() => setRevision((value) => value + 1)}
            >
              Réessayer
            </button>
          )}
        </div>
      )}
    </section>
  );
}
