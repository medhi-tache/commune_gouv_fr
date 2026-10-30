import type { Metadata } from "next";
import { ArrowUpRight, Flag } from "lucide-react";
import Dashboard from "../../components/dashboard";
import GovernmentDemoAccount from "../../components/government-demo-account";
import presidency from "../../reference/presidency.json";
import "../gouvernement/government.css";

export const metadata: Metadata = {
  title: "Présidence de la République — Qomune",
  description:
    "Fiche du président de la République et compte fictif de démonstration pour publier dans les communes.",
};

/** La Présidence est présentée séparément de la composition du Gouvernement. */
export default function PresidencyPage() {
  const person = presidency.member;
  const checkedDate = new Intl.DateTimeFormat("fr-FR", {
    dateStyle: "long",
    timeZone: "UTC",
  }).format(new Date(`${presidency.checked_at}T00:00:00Z`));
  return (
    <Dashboard section="presidence">
      <article className="government-page government-profile">
        <div className="government-profile-header">
          <span
            className="government-avatar government-avatar-large"
            aria-hidden="true"
          >
            EM
          </span>
          <div>
            <div className="eyebrow">
              <span /> PRÉSIDENCE DE LA RÉPUBLIQUE
            </div>
            <h1>{person.name}</h1>
            <p className="government-profile-role">{person.role}</p>
          </div>
        </div>
        <section className="government-profile-details">
          <h2>
            <Flag size={21} /> Chef de l’État
          </h2>
          <p>
            Le président de la République exerce une fonction distincte de celle
            des membres du Gouvernement. Il nomme le Premier ministre et préside
            le Conseil des ministres.
          </p>
          <dl>
            <dt>Fonction</dt>
            <dd>{person.role}</dd>
            <dt>Institution</dt>
            <dd>Présidence de la République</dd>
            <dt>Date de consultation</dt>
            <dd>{checkedDate}</dd>
          </dl>
          <a
            className="button government-official-link"
            href={person.official_url}
            target="_blank"
            rel="noopener noreferrer"
          >
            Consulter le profil officiel <ArrowUpRight size={17} />
            <span className="sr-only">
              {" "}
              sur le site de l’Élysée (nouvel onglet)
            </span>
          </a>
        </section>
        <GovernmentDemoAccount member={person} institution="presidence" />
        <p className="government-note">
          Source :{" "}
          <a
            href={presidency.source_url}
            target="_blank"
            rel="noopener noreferrer"
          >
            fiche officielle de l’Élysée
          </a>
          , consultée le {checkedDate}. Qomune est un prototype indépendant ; ce
          compte fictif ne représente pas la Présidence et ses publications ne
          constituent pas des communications officielles.
        </p>
      </article>
    </Dashboard>
  );
}
