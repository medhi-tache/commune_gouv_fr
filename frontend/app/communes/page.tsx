import CommuneDirectory from "../../components/commune-directory";
export const metadata = { title: "Fiches des communes — Qomune" };
export default function CommunesPage() {
  return (
    <article className="government-page commune-profile">
      <p className="eyebrow">VOTRE TERRITOIRE</p>
      <h1>Les fiches des communes</h1>
      <p>
        Retrouvez votre commune, ses publications et les carburants les moins
        chers dans ses stations.
      </p>
      <CommuneDirectory />
    </article>
  );
}
