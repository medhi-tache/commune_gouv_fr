import snapshot from "../reference/government.json";

/** Photographie datée de la composition officielle ; aucune fonction n'est déduite. */
export const government = snapshot;
export type GovernmentMember = (typeof snapshot.members)[number];
export const governmentCheckedDate = new Intl.DateTimeFormat("fr-FR", {
  dateStyle: "long",
  timeZone: "UTC",
}).format(new Date(`${snapshot.checked_at}T00:00:00Z`));

export function governmentInitials(name: string) {
  return name
    .split(" ")
    .map((part) => part[0])
    .slice(0, 2)
    .join("");
}

/** La recherche ignore accents, ponctuation et casse, pour les noms et fonctions. */
export function normalizeGovernmentSearch(value: string) {
  return value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLocaleLowerCase("fr-FR")
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}
