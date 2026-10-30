import GovernmentDemoAccount from "../../../components/government-demo-account";
import type { Metadata } from "next";
import { socialMetadata } from "../../../lib/social-metadata";
import { notFound } from "next/navigation";
import { ArrowLeft, ArrowUpRight, Landmark } from "lucide-react";
import {
  government,
  governmentCheckedDate,
  governmentInitials,
} from "../../../lib/government";

type ProfileProps = { params: Promise<{ slug: string }> };

export function generateStaticParams() {
  return government.members.map(({ slug }) => ({ slug }));
}

export async function generateMetadata({
  params,
}: ProfileProps): Promise<Metadata> {
  const { slug } = await params;
  const member = government.members.find((item) => item.slug === slug);
  if (!member) return { title: "Profil introuvable — Qomune" };
  return socialMetadata(
    `${member.name} — Gouvernement · Qomune`,
    `Fiche publique de ${member.name} : ${member.role}. Prototype Qomune, sans caractère officiel.`,
    `/gouvernement/${member.slug}`,
  );
}

/** Les profils ne créent aucun compte et ne permettent pas de publier au nom d'un ministre. */
export default async function GovernmentProfile({ params }: ProfileProps) {
  const { slug } = await params;
  const member = government.members.find((item) => item.slug === slug);
  if (!member) notFound();

  return (
    <article className="government-page government-profile">
      <a className="government-back" href="/gouvernement">
        <ArrowLeft size={16} /> Tous les membres du Gouvernement
      </a>
      <div className="government-profile-header">
        <span
          className="government-avatar government-avatar-large"
          aria-hidden="true"
        >
          {governmentInitials(member.name)}
        </span>
        <div>
          <div className="eyebrow">
            <span /> MEMBRE DU GOUVERNEMENT
          </div>
          <h1>{member.name}</h1>
          <p className="government-profile-role">{member.role}</p>
        </div>
      </div>
      <section className="government-profile-details">
        <h2>
          <Landmark size={21} /> Fonction et rattachement
        </h2>
        <dl>
          <dt>Fonction</dt>
          <dd>{member.role}</dd>
          <dt>Rattachement{member.ministries.length > 1 ? "s" : ""}</dt>
          <dd>
            <ul>
              {member.ministries.map((ministry) => (
                <li key={ministry}>{ministry}</li>
              ))}
            </ul>
          </dd>
          <dt>Date de consultation</dt>
          <dd>{governmentCheckedDate}</dd>
        </dl>
        <a
          className="button government-official-link"
          href={member.official_url}
          target="_blank"
          rel="noopener noreferrer"
        >
          Consulter le profil officiel <ArrowUpRight size={17} />
          <span className="sr-only"> sur info.gouv.fr (nouvel onglet)</span>
        </a>
      </section>
      <GovernmentDemoAccount member={member} />
      <p className="government-note">
        Source :{" "}
        <a
          href={government.source_url}
          target="_blank"
          rel="noopener noreferrer"
        >
          composition officielle du Gouvernement sur info.gouv.fr
        </a>
        . Cette fiche reprend les informations consultées le{" "}
        {governmentCheckedDate}. Les fonctions peuvent évoluer.
      </p>
    </article>
  );
}
