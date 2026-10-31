import { notFound } from "next/navigation";
import { ArrowLeft, ArrowUpRight, MapPin } from "lucide-react";
import FuelPrices from "../../../components/fuel-prices";
import type { Commune } from "../../../components/commune-picker";

// Une fiche est résolue à la demande depuis le référentiel Qomune, y compris pour la Corse et l’outre-mer.
export const dynamic = "force-dynamic";
export default async function CommunePage({
  params,
}: {
  params: Promise<{ code: string }>;
}) {
  const { code } = await params;
  if (!/^(?:[0-9]{5}|2[AB][0-9]{3})$/.test(code)) notFound();
  const response = await fetch(
    `${process.env.API_BASE_URL || "http://127.0.0.1:8080"}/communes?ids=${encodeURIComponent(code)}`,
    {
      cache: "no-store",
      signal: AbortSignal.timeout(15000),
    },
  );
  if (!response.ok)
    throw new Error(
      "Le référentiel des communes est momentanément indisponible.",
    );
  const communes: Commune[] = await response.json();
  const commune = communes.find((item) => item.id === code);
  if (!commune) notFound();
  return (
    <article className="government-page commune-profile">
      <a className="government-back" href="/communes">
        <ArrowLeft size={16} /> Toutes les communes
      </a>
      <p className="eyebrow">FICHE DE LA COMMUNE</p>
      <h1>
        <MapPin size={30} aria-hidden="true" /> {commune.name}
      </h1>
      <dl className="commune-facts">
        <div>
          <dt>Département</dt>
          <dd>{commune.department}</dd>
        </div>
        <div>
          <dt>Région</dt>
          <dd>{commune.region}</dd>
        </div>
        <div>
          <dt>Code INSEE</dt>
          <dd>{commune.id}</dd>
        </div>
      </dl>
      <a className="button" href={`/?commune=${commune.id}`}>
        Voir les publications de {commune.name} <ArrowUpRight size={16} />
      </a>
      <FuelPrices communeId={commune.id} communeName={commune.name} />
      <p className="government-note">
        Fiche informative du prototype indépendant Qomune. Les prix proviennent
        des déclarations des stations ; cette page ne constitue pas le site
        officiel de la commune.
      </p>
    </article>
  );
}
