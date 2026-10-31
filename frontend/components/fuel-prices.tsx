"use client";

import { useEffect, useState } from "react";
import { ArrowUpRight, Fuel, RefreshCw } from "lucide-react";
import { api } from "../lib/api";

type Station = {
  id: string;
  address: string;
  city: string;
  updated_at: string;
  older_than_week: boolean;
  distance_km: number | null;
};
type Prices = {
  commune_id: string;
  station_count: number;
  fetched_at: string;
  source_url: string;
  prices: {
    fuel: string;
    search_scope: "commune" | "nearby" | "none";
    price_per_litre: number | null;
    stations: Station[];
  }[];
};
const dateLabel = (date: string) =>
  new Intl.DateTimeFormat("fr-FR", {
    dateStyle: "short",
    timeStyle: "short",
    timeZone: "Europe/Paris",
  }).format(new Date(date));

/** Comparaison publique par carburant ; aucune géolocalisation du visiteur n’est demandée. */
export default function FuelPrices({
  communeId,
  communeName,
}: {
  communeId: string;
  communeName: string;
}) {
  const [data, setData] = useState<Prices | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    let current = true;
    const load = async () => {
      setLoading(true);
      try {
        const result = await api<Prices>(
          `carburants/${encodeURIComponent(communeId)}`,
        );
        if (current) {
          setData(result);
          setError("");
        }
      } catch (error) {
        if (current) setError((error as Error).message);
      } finally {
        if (current) setLoading(false);
      }
    };
    void load();
    const timer = window.setInterval(() => void load(), 600000);
    return () => {
      current = false;
      window.clearInterval(timer);
    };
  }, [communeId, revision]);
  const hasPrices = data?.prices.some((item) => item.price_per_litre !== null);
  return (
    <section
      className="fuel-prices"
      aria-labelledby="fuel-title"
      aria-busy={loading}
    >
      <div className="fuel-heading">
        <div>
          <p className="fuel-eyebrow">PRATIQUE · VOTRE COMMUNE</p>
          <h2 id="fuel-title">
            <Fuel size={23} /> Le carburant le moins cher à {communeName}
          </h2>
          <p>
            Les prix les plus bas dans la commune. Pour chaque carburant sans
            prix local, recherche dans un rayon de 25 km autour du centre
            approximatif de la commune.
          </p>
        </div>
      </div>
      {loading && !data && (
        <p role="status">Recherche des prix officiels des stations…</p>
      )}
      {error && (
        <div className="fuel-error" role="alert">
          <p>
            {data
              ? "L’actualisation a échoué. Les derniers relevés ci-dessous restent datés et sont à vérifier."
              : error}
          </p>
          <button
            className="button"
            disabled={loading}
            onClick={() => setRevision((value) => value + 1)}
          >
            <RefreshCw size={15} /> Réessayer
          </button>
        </div>
      )}
      {data && !hasPrices && (
        <p className="fuel-empty">
          Aucun prix disponible dans la commune ni dans un rayon de 25 km autour
          de son centre approximatif.
        </p>
      )}
      {data && hasPrices && (
        <div className="fuel-grid">
          {data.prices.map((item) => (
            <article
              className={`fuel-card${item.price_per_litre === null ? " fuel-missing" : ""}`}
              key={item.fuel}
            >
              <h3>{item.fuel}</h3>
              <p
                className={
                  item.search_scope === "nearby" ? "fuel-nearby" : "fuel-area"
                }
              >
                {item.search_scope === "nearby"
                  ? "À proximité · aucun prix local"
                  : item.search_scope === "commune"
                    ? "Dans la commune"
                    : "Commune et rayon de 25 km"}
              </p>
              {item.price_per_litre === null ? (
                <p>Aucun prix disponible</p>
              ) : (
                <>
                  <p className="fuel-price">
                    {item.price_per_litre.toLocaleString("fr-FR", {
                      minimumFractionDigits: 3,
                      maximumFractionDigits: 3,
                    })}
                    <span> €/L</span>
                  </p>
                  {item.stations.length > 1 && (
                    <p className="fuel-tie">
                      {item.stations.length} stations au même prix
                    </p>
                  )}
                  <ul>
                    {item.stations.map((station) => (
                      <li key={station.id}>
                        <strong>{station.address}</strong>
                        <span>{station.city}</span>
                        {station.distance_km != null && (
                          <small>
                            À environ{" "}
                            {station.distance_km.toLocaleString("fr-FR")} km à
                            vol d’oiseau du centre approximatif de {communeName}
                          </small>
                        )}
                        <small>
                          Prix déclaré le{" "}
                          <time dateTime={station.updated_at}>
                            {dateLabel(station.updated_at)}
                          </time>
                        </small>
                        {station.older_than_week && (
                          <small className="fuel-old">
                            Déclaration de plus de 7 jours · à vérifier
                          </small>
                        )}
                      </li>
                    ))}
                  </ul>
                </>
              )}
            </article>
          ))}
        </div>
      )}
      <div className="fuel-source">
        <a
          href="https://www.data.gouv.fr/datasets/prix-des-carburants-en-france-flux-instantane-v2-amelioree"
          target="_blank"
          rel="noopener noreferrer"
        >
          Source : prix-carburants.gouv.fr / data.gouv.fr{" "}
          <ArrowUpRight size={14} />
          <span className="sr-only"> (nouvel onglet)</span>
        </a>
        {data && (
          <span>
            {data.station_count} station{data.station_count > 1 ? "s" : ""}{" "}
            référencée{data.station_count > 1 ? "s" : ""} · Données récupérées
            le {dateLabel(data.fetched_at)}
          </span>
        )}
      </div>
      <p className="fuel-disclaimer">
        Les données sont relues toutes les 10 minutes. La date de déclaration de
        chaque prix est distincte. Les carburants déclarés en rupture sont
        exclus ; disponibilité et prix à confirmer en station. Les distances
        sont indicatives et ne correspondent pas à un trajet routier.
      </p>
    </section>
  );
}
