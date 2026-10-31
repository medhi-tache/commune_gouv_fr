"use client";

import { useState } from "react";
import { ArrowRight, ArrowUpRight, Landmark, Search } from "lucide-react";
import { prefectures, prefecturesCheckedDate } from "../lib/prefectures";
import { normalizeGovernmentSearch } from "../lib/government";

/** Une fiche et un compte fictif par préfecture départementale, Paris et outre-mer compris. */
export default function PrefectureDirectory() {
  const [query, setQuery] = useState("");
  const terms = normalizeGovernmentSearch(query).split(" ").filter(Boolean);
  const members = prefectures.members.filter((item) => {
    const text = normalizeGovernmentSearch(
      `${item.name} ${item.department_code} ${item.role}`,
    );
    return terms.every((term) => text.includes(term));
  });
  return (
    <div className="government-page">
      <section className="government-heading">
        <div className="eyebrow">
          <span /> LES INSTITUTIONS, EN LIEN DIRECT
        </div>
        <h1>Les préfectures.</h1>
        <p>
          101 préfectures départementales, en métropole et outre-mer. Retrouvez
          leur profil et les publications de leur compte fictif.
        </p>
        <div className="government-source">
          <span>Annuaire consulté le {prefecturesCheckedDate}</span>
          <a
            href={prefectures.source_url}
            target="_blank"
            rel="noopener noreferrer"
          >
            Source : Service Public / DILA <ArrowUpRight size={14} />
          </a>
        </div>
      </section>
      <section
        className="government-intro"
        aria-label="Présentation des préfectures"
      >
        <div className="government-emblem">
          <Landmark size={29} />
        </div>
        <div>
          <h2>Un département, un espace de publication.</h2>
          <p>
            Chaque compte fictif peut diffuser vers toutes les communes de son
            département ou une seule commune de ce territoire.
          </p>
        </div>
        <span className="government-status">Comptes fictifs</span>
      </section>
      <div className="government-toolbar">
        <label className="government-search">
          <Search size={18} />
          <span className="sr-only">
            Rechercher une préfecture ou un département
          </span>
          <input
            type="search"
            placeholder="Rechercher un département, un numéro…"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
        </label>
        <p role="status">
          {members.length} préfecture{members.length > 1 ? "s" : ""}
        </p>
      </div>
      <div className="government-grid">
        {members.map((item) => (
          <a
            className="government-card"
            href={`/prefectures/${item.slug}`}
            key={item.slug}
          >
            <div className="government-card-top">
              <span className="government-avatar" aria-hidden="true">
                {item.department_code}
              </span>
              <span className="government-kind">Préfecture</span>
            </div>
            <h2>{item.department_name}</h2>
            <p className="government-role">
              {item.communes_count.toLocaleString("fr-FR")} commune
              {item.communes_count > 1 ? "s" : ""} · Compte de démonstration
            </p>
            <span className="government-card-action">
              Voir le profil <ArrowRight size={16} />
            </span>
          </a>
        ))}
      </div>
      {!members.length && (
        <div className="government-empty">
          <h2>Aucune préfecture trouvée</h2>
          <button className="button" onClick={() => setQuery("")}>
            Afficher toutes les préfectures
          </button>
        </div>
      )}
      <p className="government-note">
        Qomune est un prototype indépendant. Ces comptes n’appartiennent pas aux
        préfectures et ne constituent pas des canaux officiels de l’État.
      </p>
    </div>
  );
}
