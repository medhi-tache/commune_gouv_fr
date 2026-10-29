export default function NotFound() {
  return (
    <main
      className="empty-state"
      style={{ margin: "10vh auto", maxWidth: 600 }}
    >
      <h1>Cette page est introuvable.</h1>
      <p>Retrouvez les publications sur le fil des communes.</p>
      <a className="button primary" href="/">
        Revenir à l’accueil
      </a>
    </main>
  );
}
