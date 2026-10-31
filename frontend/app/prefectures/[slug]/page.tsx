import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ArrowLeft, ArrowUpRight, Landmark } from "lucide-react";
import { prefectures, prefecturesCheckedDate } from "../../../lib/prefectures";
import { socialMetadata } from "../../../lib/social-metadata";
import GovernmentDemoAccount from "../../../components/government-demo-account";
import GovernmentPublications from "../../../components/government-publications";

type ProfileProps = { params: Promise<{ slug: string }> };
export function generateStaticParams() {
  return prefectures.members.map(({ slug }) => ({ slug }));
}
export async function generateMetadata({
  params,
}: ProfileProps): Promise<Metadata> {
  const { slug } = await params;
  const member = prefectures.members.find((item) => item.slug === slug);
  if (!member) return { title: "Préfecture introuvable — Qomune" };
  return socialMetadata(
    `${member.name} — Qomune`,
    `Profil et publications du compte fictif de démonstration : ${member.name}.`,
    `/prefectures/${slug}`,
  );
}

/** Les informations officielles et le compte de test restent clairement distingués. */
export default async function PrefectureProfile({ params }: ProfileProps) {
  const { slug } = await params;
  const member = prefectures.members.find((item) => item.slug === slug);
  if (!member) notFound();
  return (
    <article className="government-page government-profile">
      <a className="government-back" href="/prefectures">
        <ArrowLeft size={16} /> Toutes les préfectures
      </a>
      <div className="government-profile-header">
        <span
          className="government-avatar government-avatar-large"
          aria-hidden="true"
        >
          {member.department_code}
        </span>
        <div>
          <div className="eyebrow">
            <span /> PRÉFECTURE DÉPARTEMENTALE
          </div>
          <h1>{member.name}</h1>
          <p className="government-profile-role">{member.role}</p>
        </div>
      </div>
      <section className="government-profile-details">
        <h2>
          <Landmark size={21} /> Territoire et institution
        </h2>
        <dl>
          <dt>Département</dt>
          <dd>
            {member.department_name} ({member.department_code})
          </dd>
          <dt>Communes couvertes</dt>
          <dd>
            {member.communes_count.toLocaleString("fr-FR")} · référentiel Insee
            COG 2026
          </dd>
          <dt>Date de consultation</dt>
          <dd>{prefecturesCheckedDate}</dd>
        </dl>
        <a
          className="button government-official-link"
          href={member.official_url}
          target="_blank"
          rel="noopener noreferrer"
        >
          Consulter le site officiel <ArrowUpRight size={17} />
          <span className="sr-only"> (nouvel onglet)</span>
        </a>
      </section>
      <GovernmentPublications
        slug={member.slug}
        name={member.name}
        institution="prefectures"
      />
      <GovernmentDemoAccount member={member} institution="prefecture" />
      <p className="government-note">
        Source :{" "}
        <a href={member.source_url} target="_blank" rel="noopener noreferrer">
          annuaire officiel Service Public (nouvel onglet)
        </a>
        . Ce compte fictif ne représente pas la préfecture. L’accès des
        institutions réelles nécessitera une vérification préalable.
      </p>
    </article>
  );
}
