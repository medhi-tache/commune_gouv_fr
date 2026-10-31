import snapshot from "../reference/prefectures.json";

/** Annuaire institutionnel DILA ; aucun nom de préfet ni mandat n’est déduit. */
export const prefectures = snapshot;
export const prefecturesCheckedDate = new Intl.DateTimeFormat("fr-FR", {
  dateStyle: "long",
  timeZone: "UTC",
}).format(new Date(`${snapshot.checked_at}T00:00:00Z`));
