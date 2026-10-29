"use client";

import { useEffect, useId, useRef, useState, type CSSProperties } from "react";
import { Check, ChevronDown, MapPin, Search } from "lucide-react";
import { api } from "../lib/api";

export type Commune = {
  id: string;
  name: string;
  department: string;
  region: string;
};

/** Recherche distante bornée : le navigateur ne crée jamais 34 875 options HTML. */
export default function CommunePicker({
  value,
  onChange,
  label,
  disabled = false,
  compact = false,
}: {
  value: Commune | null;
  onChange: (commune: Commune | null) => void;
  label: string;
  disabled?: boolean;
  compact?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Commune[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const root = useRef<HTMLDivElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const popupId = useId();
  const [popupStyle, setPopupStyle] = useState<CSSProperties>({});

  // Le panneau reste dans l'écran, même si le filtre est proche du bas de la fenêtre.
  function positionPopup() {
    const box = trigger.current?.getBoundingClientRect();
    if (!box) return;
    const width = Math.min(340, window.innerWidth - 32);
    const below = window.innerHeight - box.bottom - 24;
    const above = box.top - 24;
    const upwards = below < 300 && above > below;
    setPopupStyle({
      left: Math.max(16, Math.min(box.left, window.innerWidth - width - 16)),
      width,
      top: upwards ? undefined : Math.max(16, box.bottom + 7),
      bottom: upwards ? window.innerHeight - box.top + 7 : undefined,
      maxHeight: Math.min(400, Math.max(180, upwards ? above : below)),
    });
  }

  useEffect(() => {
    if (!open) return;
    positionPopup();
    input.current?.focus();
    const outside = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", outside);
    window.addEventListener("resize", positionPopup);
    return () => {
      document.removeEventListener("pointerdown", outside);
      window.removeEventListener("resize", positionPopup);
    };
  }, [open]);

  useEffect(() => {
    if (!open) return;
    let current = true;
    setLoading(true);
    setResults([]);
    setError("");
    const timer = setTimeout(() => {
      api<Commune[]>(`communes?limit=20&q=${encodeURIComponent(query)}`)
        .then((items) => {
          if (current) setResults(items);
        })
        .catch((error) => {
          if (current) setError(error.message);
        })
        .finally(() => {
          if (current) setLoading(false);
        });
    }, 200);
    return () => {
      current = false;
      clearTimeout(timer);
    };
  }, [open, query]);

  function choose(commune: Commune | null) {
    onChange(commune);
    setOpen(false);
    setQuery("");
    trigger.current?.focus();
  }

  return (
    <div
      className={`commune-picker ${compact ? "compact" : ""}`}
      ref={root}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null))
          setOpen(false);
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape" && open) {
          event.preventDefault();
          event.stopPropagation();
          setOpen(false);
          trigger.current?.focus();
        }
      }}
    >
      <button
        ref={trigger}
        className="commune-trigger"
        aria-label={label}
        aria-expanded={open}
        aria-controls={popupId}
        disabled={disabled}
        onClick={() => setOpen(!open)}
      >
        <MapPin size={17} />
        <span>{value?.name || "Toutes les communes"}</span>
        <ChevronDown size={15} />
      </button>
      {open && (
        <div id={popupId} className="commune-popup" style={popupStyle}>
          <label className="commune-search">
            <Search size={16} />
            <input
              ref={input}
              aria-label="Rechercher une commune"
              placeholder="Nom ou code INSEE…"
              maxLength={100}
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
          </label>
          <button
            className="commune-result all-communes"
            onClick={() => choose(null)}
          >
            Toutes les communes {!value && <Check size={14} />}
          </button>
          <div className="commune-results" aria-live="polite">
            {loading ? (
              <p role="status">Recherche en cours…</p>
            ) : error ? (
              <p role="alert">{error}</p>
            ) : !results.length ? (
              <p>Aucune commune trouvée.</p>
            ) : (
              results.map((commune) => (
                <button
                  className="commune-result"
                  key={commune.id}
                  onClick={() => choose(commune)}
                >
                  <span>
                    <strong>{commune.name}</strong>
                    <small>
                      {commune.department} · INSEE {commune.id}
                    </small>
                  </span>
                  {value?.id === commune.id && <Check size={14} />}
                </button>
              ))
            )}
          </div>
          <p className="commune-search-note">
            20 résultats maximum · Affinez le nom ou le code INSEE.
          </p>
        </div>
      )}
    </div>
  );
}
