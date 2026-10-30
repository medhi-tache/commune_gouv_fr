"use client";

import { useState } from "react";
import { ArrowRight, ArrowUpRight, Landmark, Search } from "lucide-react";
import {
  government,
  governmentCheckedDate,
  governmentInitials,
  normalizeGovernmentSearch,
} from "../lib/government";

/** Annuaire public : une fiche par personne, y compris en cas de double rattachement. */
export default function GovernmentDirectory() {
  const [query, setQuery] = useState("");
  const terms = normalizeGovernmentSearch(query).split(" ").filter(Boolean);
  const members = government.members.filter((member) => {
    const text = normalizeGovernmentSearch(
      [member.name, member.role, ...member.ministries].join(" "),
    );
    return terms.every((term) => text.includes(term));
  });

  return (
    <div className="government-page">
      <section className="government-heading">
        <div className="eyebrow">
          <span /> LES INSTITUTIONS, EN LIEN DIRECT
        </div>
        <h1>Le Gouvernement.</h1>
        <p>
          Les membres du Gouvernement et leurs fonctions, réunis dans un
          annuaire public.
        </p>
        <div className="government-source">
          <span>Composition consultée le {governmentCheckedDate}</span>
          <a
            href={government.source_url}
            target="_blank"
            rel="noopener noreferrer"
          >
            Source : info.gouv.fr <ArrowUpRight size={14} />
          </a>
        </div>
      </section>
      <section
        className="government-intro"
        aria-label="Présentation de l’annuaire"
      >
        <div className="government-emblem">
          <Landmark size={29} />
        </div>
        <div>
          <h2>{government.members.length} membres, au service de l’État.</h2>
          <p>
            Consultez une fiche pour retrouver la fonction complète, le
            rattachement ministériel et le profil officiel.
          </p>
        </div>
        <span className="government-status">Composition référencée</span>
      </section>
      <div className="government-toolbar">
        <label className="government-search">
          <Search size={18} />
          <span className="sr-only">Rechercher un membre ou une fonction</span>
          <input
            type="search"
            placeholder="Rechercher un nom, une fonction…"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
        </label>
        <p role="status">
          {members.length} membre{members.length > 1 ? "s" : ""}
        </p>
      </div>
      <div className="government-grid">
        {members.map((member) => (
          <a
            className="government-card"
            href={`/gouvernement/${member.slug}`}
            key={member.slug}
          >
            <div className="government-card-top">
              <span className="government-avatar" aria-hidden="true">
                {governmentInitials(member.name)}
              </span>
              <span className="government-kind">
                {member.slug === "sebastien-lecornu"
                  ? "Premier ministre"
                  : member.role.startsWith("Ministre délégu")
                    ? "Ministre délégué·e"
                    : "Ministre"}
              </span>
            </div>
            <h2>{member.name}</h2>
            <p className="government-role">{member.role}</p>
            <span className="government-card-action">
              Voir le profil <ArrowRight size={16} />
            </span>
          </a>
        ))}
      </div>
      {!members.length && (
        <div className="government-empty">
          <h2>Aucun membre trouvé</h2>
          <p>Essayez un autre nom ou une autre fonction.</p>
          <button className="button" onClick={() => setQuery("")}>
            Afficher tous les membres
          </button>
        </div>
      )}
      <p className="government-note">
        Cet annuaire reprend la composition publiée par info.gouv.fr à la date
        indiquée. Les fonctions peuvent évoluer ; consultez la source officielle
        pour les changements ultérieurs.
      </p>
    </div>
  );
}
