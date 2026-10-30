/** Statistiques anonymes visibles uniquement après la connexion administrateur. */
export type VisitMetrics = {
  started_at: string;
  total: number;
  timeline: {
    date: string;
    public: number;
    admin_demo: number;
    admin_private: number;
    measured: boolean;
  }[];
};
const format = (value: number) => value.toLocaleString("fr-FR");
const label = (date: string) =>
  new Date(`${date}T12:00:00Z`).toLocaleDateString("fr-FR", {
    day: "numeric",
    month: "short",
    timeZone: "UTC",
  });
const count = (day: VisitMetrics["timeline"][number]) =>
  day.public + day.admin_demo + day.admin_private;
export default function VisitsChart({
  visits,
  days,
}: {
  visits: VisitMetrics;
  days: number;
}) {
  const max = Math.max(
    3,
    Math.ceil(Math.max(0, ...visits.timeline.map(count)) / 3) * 3,
  );
  const totals = visits.timeline.reduce(
    (acc, day) => ({
      public: acc.public + day.public,
      demo: acc.demo + day.admin_demo,
      private: acc.private + day.admin_private,
    }),
    { public: 0, demo: 0, private: 0 },
  );
  return (
    <section
      className="admin-panel admin-visits"
      aria-labelledby="visits-title"
    >
      <div className="admin-panel-heading">
        <div>
          <h3 id="visits-title">Connexions au site</h3>
          <p className="admin-panel-subtitle">
            Pages vues par tous les visiteurs, connectés ou non · {days} jours ·
            UTC
          </p>
        </div>
        <span>
          <strong>{format(visits.total)}</strong> pages vues
        </span>
      </div>
      <svg
        className="admin-plot"
        viewBox="0 0 760 220"
        role="img"
        aria-label={`${format(visits.total)} pages vues sur la période. Le détail quotidien est disponible dans le tableau ci-dessous.`}
      >
        {[0, 1, 2, 3].map((n) => (
          <g key={n}>
            <line
              x1="35"
              x2="745"
              y1={180 - n * 50}
              y2={180 - n * 50}
              stroke="#e6eaf3"
            />
            <text
              x="25"
              y={184 - n * 50}
              textAnchor="end"
              fontSize="11"
              fill="#68738c"
            >
              {(max * n) / 3}
            </text>
          </g>
        ))}
        {visits.timeline.map((day, index) => {
          const width = 700 / days,
            x = 40 + index * width,
            height = (count(day) / max) * 150;
          return (
            <g key={day.date}>
              <title>
                {label(day.date)} :{" "}
                {day.measured
                  ? `${count(day)} pages vues`
                  : "mesure non disponible"}
              </title>
              {!day.measured ? (
                <rect x={x} y="30" width={width} height="150" fill="#f0f2f7" />
              ) : (
                <rect
                  x={x + width * 0.17}
                  y={180 - height}
                  width={width * 0.66}
                  height={height}
                  rx="2"
                  fill="#002395"
                />
              )}
              {(index === 0 ||
                index === days - 1 ||
                index === Math.floor(days / 2)) && (
                <text
                  x={x + width / 2}
                  y="207"
                  textAnchor={
                    index === 0
                      ? "start"
                      : index === days - 1
                        ? "end"
                        : "middle"
                  }
                  fontSize="11"
                  fill="#68738c"
                >
                  {label(day.date)}
                </text>
              )}
            </g>
          );
        })}
      </svg>
      <div className="admin-visit-totals">
        <span>
          Fil public <strong>{format(totals.public)}</strong>
        </span>
        <span>
          Démo administration <strong>{format(totals.demo)}</strong>
        </span>
        <span>
          Accès administration privée <strong>{format(totals.private)}</strong>
        </span>
      </div>
      <p className="admin-panel-subtitle admin-visit-note">
        Chaque chargement de page compte pour une visite, y compris un
        rechargement. Ce compteur ne mesure ni les personnes uniques ni les
        connexions à un compte. Les robots peuvent être inclus. Aucun cookie de
        suivi, adresse IP ou email n’est enregistré pour cette mesure.
      </p>
      <p className="admin-panel-subtitle admin-visit-note">
        Suivi commencé le{" "}
        {visits.started_at
          ? new Date(visits.started_at).toLocaleString("fr-FR", {
              timeZone: "UTC",
            })
          : "début de cette mise à jour"}{" "}
        (UTC). Les zones grises correspondent à une période non mesurée ;
        l’historique antérieur n’est pas reconstitué.
      </p>
      <details className="admin-details">
        <summary>Consulter les visites par jour</summary>
        <div className="admin-table-scroll">
          <table>
            <caption>Pages vues par jour (UTC)</caption>
            <thead>
              <tr>
                <th>Date</th>
                <th>Pages vues</th>
              </tr>
            </thead>
            <tbody>
              {visits.timeline.map((day) => (
                <tr key={day.date}>
                  <th scope="row">{label(day.date)}</th>
                  <td>{day.measured ? format(count(day)) : "Non mesuré"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </details>
    </section>
  );
}
