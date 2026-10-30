"use client";

import { useEffect, useState, type FormEvent } from "react";
import {
  ArrowLeft,
  ArrowUpRight,
  BarChart3,
  Building2,
  FileText,
  LockKeyhole,
  LogOut,
  RefreshCw,
  ShieldCheck,
  Users,
} from "lucide-react";
import { api } from "../lib/api";

type Statistic = { name: string; count: number };
type Statistics = {
  days: number;
  generated_at: string;
  totals: {
    communes: number;
    registered_communes: number;
    members: number;
    mayors: number;
    public: number;
    private: number;
    comments: number;
    reactions: number;
  };
  timeline: { date: string; public: number; private: number }[];
  categories: Statistic[];
  top_communes: Statistic[];
};
const format = (value: number) => value.toLocaleString("fr-FR");
const dateLabel = (date: string) =>
  new Date(`${date}T12:00:00Z`).toLocaleDateString("fr-FR", {
    day: "numeric",
    month: "short",
    timeZone: "UTC",
  });

/** La démonstration publique n'interroge jamais l'API des statistiques réelles. */
export default function Administration({ demo = false }: { demo?: boolean }) {
  const [data, setData] = useState<Statistics | null>(null);
  const [days, setDays] = useState(30);
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [authenticated, setAuthenticated] = useState(false);
  const [checking, setChecking] = useState(true);

  async function load(period: number) {
    setBusy(true);
    setError("");
    try {
      const result = demo
        ? demonstration(period)
        : await api<Statistics>(`admin/statistics?days=${period}`);
      setData(result);
      setDays(period);
      setAuthenticated(true);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Chargement impossible.";
      setError(message);
      if (message === "Connectez-vous à l’administration.") {
        setData(null);
        setAuthenticated(false);
      }
    } finally {
      setBusy(false);
      setChecking(false);
    }
  }
  useEffect(() => {
    void load(30);
  }, []);
  async function login(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      await api("admin/login", { password });
      setPassword("");
      await load(days);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Connexion impossible.");
    } finally {
      setBusy(false);
    }
  }
  async function logout() {
    setBusy(true);
    setError("");
    try {
      await api("admin/logout", {});
      setData(null);
      setAuthenticated(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Déconnexion impossible.");
    } finally {
      setBusy(false);
    }
  }
  const totalPosts = data ? data.totals.public + data.totals.private : 0;
  const periodPosts =
    data?.timeline.reduce((n, day) => n + day.public + day.private, 0) || 0;
  const maxDay = Math.max(
    3,
    Math.ceil(
      Math.max(
        0,
        ...(data?.timeline.map((day) => day.public + day.private) || []),
      ) / 3,
    ) * 3,
  );
  const publicShare = totalPosts ? (data!.totals.public / totalPosts) * 100 : 0;

  return (
    <div className="admin-shell">
      <aside className="admin-sidebar">
        <a className="admin-brand" href="/">
          qomune<span>.</span>
        </a>
        <span className="admin-caption">PILOTAGE DE LA PLATEFORME</span>
        <a
          href={demo ? "/administration" : "/administration/connexion"}
          className="admin-nav-active"
        >
          <BarChart3 size={18} /> Vue d’ensemble
        </a>
        <div className="admin-sidebar-note">
          <ShieldCheck size={24} />
          <strong>
            {demo ? "Une démo ouverte à tous" : "Un accès réservé"}
          </strong>
          <p>
            {demo
              ? "Explorez les graphiques avec des données fictives, sans compte ni mot de passe."
              : "Les statistiques réelles sont accessibles uniquement à l’administrateur."}
          </p>
        </div>
        <a href="/" className="admin-back">
          <ArrowLeft size={16} /> Retour aux communes
        </a>
      </aside>
      <div className="admin-workspace">
        <header className="admin-topbar">
          <span>
            Administration <span className="prototype-badge">PROTOTYPE</span>
          </span>
          <span className="admin-session">
            <LockKeyhole size={14} />{" "}
            {demo ? "Démonstration publique" : "Espace protégé"}{" "}
            {demo && (
              <a
                href="/administration/connexion"
                className="specifications-link"
              >
                Administration privée
              </a>
            )}
            {authenticated && !demo && (
              <button
                onClick={logout}
                disabled={busy}
                aria-label="Se déconnecter de l’administration"
              >
                <LogOut size={17} />
              </button>
            )}
          </span>
        </header>
        <main className="admin-main">
          <div className="admin-heading">
            <div>
              <p className="admin-eyebrow">COMPRENDRE LES USAGES</p>
              <h1>
                La plateforme en un regard<span>.</span>
              </h1>
              <p>
                Suivez l’activité des communes et la vie de leurs publications.
              </p>
            </div>
            {authenticated && (
              <button
                className="admin-refresh"
                onClick={() => load(days)}
                disabled={busy}
              >
                <RefreshCw size={16} /> {busy ? "Actualisation…" : "Actualiser"}
              </button>
            )}
          </div>
          {checking ? (
            <p role="status">Vérification de votre accès…</p>
          ) : !authenticated ? (
            <section className="admin-login">
              <div className="admin-login-icon">
                <ShieldCheck size={28} />
              </div>
              <p className="admin-eyebrow">ACCÈS ADMINISTRATEUR</p>
              <h2>Bienvenue dans votre espace de pilotage</h2>
              <p>
                Connectez-vous avec le mot de passe d’administration. Les
                comptes des élus disposent de leur propre espace municipal.
              </p>
              <form onSubmit={login}>
                <label htmlFor="admin-password">
                  Mot de passe administrateur
                </label>
                <input
                  id="admin-password"
                  type="password"
                  autoComplete="current-password"
                  required
                  maxLength={200}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
                <button className="admin-primary" disabled={busy}>
                  {busy ? "Connexion…" : "Accéder au tableau de bord"}
                  <ArrowUpRight size={17} />
                </button>
              </form>
              {error && (
                <p role="alert" className="admin-error">
                  {error}
                </p>
              )}
              <small>
                Première utilisation : l’accès doit être configuré sur le
                serveur.
              </small>
            </section>
          ) : (
            data && (
              <>
                {error && (
                  <p role="alert" className="admin-error">
                    {error}
                  </p>
                )}
                <div className="admin-notice">
                  <span className="admin-live-dot" />
                  <span>
                    {demo
                      ? "Mode démonstration : toutes les statistiques affichées sont fictives. Testez librement les filtres, sans modifier de données."
                      : "Données réelles du prototype, incluant les comptes et publications de démonstration."}
                  </span>
                </div>
                <div className="admin-kpis">
                  {[
                    {
                      label: "Communes inscrites",
                      value: data.totals.registered_communes,
                      detail: `sur ${format(data.totals.communes)} communes référencées`,
                      icon: Building2,
                    },
                    {
                      label: "Membres du conseil",
                      value: data.totals.members,
                      detail: `${format(data.totals.mayors)} maire${data.totals.mayors > 1 ? "s" : ""} et ${format(data.totals.members - data.totals.mayors)} conseiller${data.totals.members - data.totals.mayors > 1 ? "s" : ""}`,
                      icon: Users,
                    },
                    {
                      label: "Publications",
                      value: totalPosts,
                      detail: `${format(data.totals.public)} publiques · ${format(data.totals.private)} privées`,
                      icon: FileText,
                    },
                    {
                      label: "Interactions internes",
                      value: data.totals.comments + data.totals.reactions,
                      detail: `${format(data.totals.comments)} commentaire${data.totals.comments > 1 ? "s" : ""} · ${format(data.totals.reactions)} réaction${data.totals.reactions > 1 ? "s" : ""}`,
                      icon: BarChart3,
                    },
                  ].map((card) => (
                    <section className="admin-kpi" key={card.label}>
                      <div>
                        <span>{card.label}</span>
                        <card.icon size={19} />
                      </div>
                      <strong>{format(card.value)}</strong>
                      <small>{card.detail}</small>
                    </section>
                  ))}
                </div>
                <div className="admin-section-heading">
                  <div>
                    <h2>L’activité des communes</h2>
                    <p>Publications créées sur la période · dates en UTC</p>
                  </div>
                  <div
                    className="admin-periods"
                    role="group"
                    aria-label="Période des graphiques"
                  >
                    {[7, 30, 90].map((n) => (
                      <button
                        key={n}
                        aria-pressed={days === n}
                        disabled={busy}
                        onClick={async () => {
                          await load(n);
                        }}
                      >
                        {n} jours
                      </button>
                    ))}
                  </div>
                </div>
                <div className="admin-charts">
                  <section className="admin-panel admin-timeline">
                    <div className="admin-panel-heading">
                      <h3>Le rythme des publications</h3>
                      <span>
                        <strong>{format(periodPosts)}</strong> sur {data.days}{" "}
                        jours
                      </span>
                    </div>
                    <div className="admin-legend">
                      <span>
                        <i /> Publiques
                      </span>
                      <span>
                        <i className="red" /> Privées
                      </span>
                    </div>
                    <svg
                      className="admin-plot"
                      viewBox="0 0 760 240"
                      role="img"
                      aria-label={`Publications quotidiennes : ${periodPosts} sur ${data.days} jours. Le tableau détaillé est disponible ci-dessous.`}
                    >
                      {[0, 1, 2, 3].map((n) => (
                        <g key={n}>
                          <line
                            x1="35"
                            x2="745"
                            y1={195 - n * 55}
                            y2={195 - n * 55}
                            stroke="#e6eaf3"
                          />
                          <text
                            x="25"
                            y={199 - n * 55}
                            textAnchor="end"
                            fill="#68738c"
                            fontSize="11"
                          >
                            {Math.ceil((maxDay * n) / 3)}
                          </text>
                        </g>
                      ))}
                      {data.timeline.map((day, index) => {
                        const width = 700 / data.days;
                        const x = 40 + index * width;
                        const pubHeight = (day.public / maxDay) * 165;
                        const privateHeight = (day.private / maxDay) * 165;
                        return (
                          <g key={day.date}>
                            <title>
                              {dateLabel(day.date)} : {day.public} publiques,{" "}
                              {day.private} privées
                            </title>
                            <rect
                              x={x + width * 0.17}
                              y={195 - pubHeight}
                              width={width * 0.66}
                              height={pubHeight}
                              rx="2"
                              fill="#002395"
                            />
                            <rect
                              x={x + width * 0.17}
                              y={195 - pubHeight - privateHeight}
                              width={width * 0.66}
                              height={privateHeight}
                              rx="2"
                              fill="#ed2939"
                            />
                            {(index === 0 ||
                              index === data.days - 1 ||
                              index === Math.floor(data.days / 2)) && (
                              <text
                                x={x + width / 2}
                                y="224"
                                fontSize="11"
                                fill="#68738c"
                                textAnchor={
                                  index === 0
                                    ? "start"
                                    : index === data.days - 1
                                      ? "end"
                                      : "middle"
                                }
                              >
                                {dateLabel(day.date)}
                              </text>
                            )}
                          </g>
                        );
                      })}
                    </svg>
                    {!periodPosts && (
                      <p className="admin-empty">
                        Aucune publication sur cette période.
                      </p>
                    )}
                    <details className="admin-details">
                      <summary>Consulter les valeurs quotidiennes</summary>
                      <div className="admin-table-scroll">
                        <table>
                          <caption>Publications par jour (UTC)</caption>
                          <thead>
                            <tr>
                              <th>Date</th>
                              <th>Publiques</th>
                              <th>Privées</th>
                            </tr>
                          </thead>
                          <tbody>
                            {data.timeline.map((day) => (
                              <tr key={day.date}>
                                <th scope="row">{dateLabel(day.date)}</th>
                                <td>{day.public}</td>
                                <td>{day.private}</td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </div>
                    </details>
                  </section>
                  <section className="admin-panel">
                    <h3>Public et conseil municipal</h3>
                    <p className="admin-panel-subtitle">
                      Ensemble des publications
                    </p>
                    <div
                      className="admin-donut"
                      role="img"
                      aria-label={`${data.totals.public} publications publiques et ${data.totals.private} privées`}
                      style={{
                        background: totalPosts
                          ? `conic-gradient(#002395 0% ${publicShare}%, #ed2939 ${publicShare}% 100%)`
                          : "#e6eaf3",
                      }}
                    >
                      <div>
                        <strong>{format(totalPosts)}</strong>
                        <span>publications</span>
                      </div>
                    </div>
                    <div className="admin-split-row">
                      <span>
                        <i /> Publiques
                      </span>
                      <strong>{format(data.totals.public)}</strong>
                    </div>
                    <div className="admin-split-row">
                      <span>
                        <i className="red" /> Privées
                      </span>
                      <strong>{format(data.totals.private)}</strong>
                    </div>
                  </section>
                  <section className="admin-panel">
                    <h3>Les thèmes qui font la vie locale</h3>
                    <p className="admin-panel-subtitle">
                      Répartition sur les {data.days} derniers jours
                    </p>
                    <Bars items={data.categories} />
                  </section>
                  <section className="admin-panel">
                    <h3>Les communes les plus actives</h3>
                    <p className="admin-panel-subtitle">
                      Publications sur les {data.days} derniers jours
                    </p>
                    <Bars items={data.top_communes} ranked />
                  </section>
                </div>
                <div className="admin-measurement">
                  <ShieldCheck size={20} />
                  <div>
                    <strong>Des indicateurs respectueux des échanges</strong>
                    <p>
                      {demo
                        ? "Cette démo est indépendante de la base du serveur. Elle ne donne accès à aucun compte ni échange privé. Les filtres et les graphiques sont utilisables librement."
                        : "Seuls les volumes sont affichés ici. Le contenu des échanges privés et les coordonnées des élus ne sont pas exposés. Les visites, visiteurs uniques et durées de consultation ne sont pas encore mesurés."}
                    </p>
                  </div>
                </div>
                <footer className="admin-footer">
                  <span>
                    Qomune ·{" "}
                    {demo
                      ? "Administration de démonstration"
                      : "Administration privée"}
                  </span>
                  <span>
                    Actualisé le{" "}
                    {new Date(data.generated_at).toLocaleString("fr-FR")}
                  </span>
                </footer>
              </>
            )
          )}
        </main>
      </div>
    </div>
  );
}
function Bars({
  items,
  ranked = false,
}: {
  items: Statistic[];
  ranked?: boolean;
}) {
  const max = Math.max(1, ...items.map((item) => item.count));
  return items.length ? (
    <ol className="admin-bars">
      {items.map((item, index) => (
        <li key={item.name}>
          <div>
            <span>
              {ranked && <em>{String(index + 1).padStart(2, "0")}</em>}
              {item.name}
            </span>
            <strong>{format(item.count)}</strong>
          </div>
          <div className="admin-bar-track">
            <span style={{ width: `${(item.count / max) * 100}%` }} />
          </div>
        </li>
      ))}
    </ol>
  ) : (
    <p className="admin-empty">Aucune publication sur cette période.</p>
  );
}

/** Jeu d'essai déterministe, indépendant des comptes et des publications du serveur. */
function demonstration(days: number): Statistics {
  const now = new Date();
  const today = Date.UTC(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate(),
  );
  const communes = [
    "Annecy",
    "Bordeaux",
    "Nantes",
    "Lyon",
    "Saint-Malo",
    "Strasbourg",
  ];
  const themes = [
    "Vie citoyenne",
    "Culture",
    "Travaux",
    "Environnement",
    "Services publics",
    "Conseil municipal",
  ];
  const timeline: Statistics["timeline"] = [];
  const categoryCounts = new Map<string, number>();
  const communeCounts = new Map<string, number>();
  let publicTotal = 0,
    privateTotal = 0;
  for (let index = 0; index < 90; index++) {
    const date = new Date(today - (89 - index) * 86400000)
      .toISOString()
      .slice(0, 10);
    const publicCount = 1 + ((index * 7 + 3) % 9);
    const privateCount = (index * 3 + 1) % 5;
    publicTotal += publicCount;
    privateTotal += privateCount;
    if (index >= 90 - days) {
      timeline.push({ date, public: publicCount, private: privateCount });
      // Les classements et les thèmes se rapportent exactement à la période choisie.
      for (let n = 0; n < publicCount + privateCount; n++) {
        const theme = themes[(index + n) % themes.length];
        const commune = communes[(index * 2 + n) % communes.length];
        categoryCounts.set(theme, (categoryCounts.get(theme) || 0) + 1);
        communeCounts.set(commune, (communeCounts.get(commune) || 0) + 1);
      }
    }
  }
  const ranking = (map: Map<string, number>) =>
    Array.from(map, ([name, count]) => ({ name, count })).sort(
      (a, b) => b.count - a.count || a.name.localeCompare(b.name, "fr"),
    );
  return {
    days,
    generated_at: now.toISOString(),
    totals: {
      communes: 34875,
      registered_communes: 6,
      members: 42,
      mayors: 6,
      public: publicTotal,
      private: privateTotal,
      comments: 168,
      reactions: 327,
    },
    timeline,
    categories: ranking(categoryCounts),
    top_communes: ranking(communeCounts).slice(0, 5),
  };
}
