"use client";

import {
  useEffect,
  useRef,
  useState,
  type FormEvent,
  type ReactNode,
} from "react";
import {
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUpRight,
  Building2,
  Check,
  ChevronDown,
  Globe2,
  Flag,
  Landmark,
  Leaf,
  LockKeyhole,
  LogOut,
  Menu,
  MessageSquare,
  Plus,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Users,
  ThumbsUp,
  X,
} from "lucide-react";

import { api } from "../lib/api";
import CouncilMembers from "./council-members";
import MayorRegistration from "./mayor-registration";
import CouncilDiscussion from "./council-discussion";
import CommunePicker, { type Commune } from "./commune-picker";

type CommuneCatalog = {
  total: number;
  vintage: number | null;
  source_url: string | null;
  imported_at: string | null;
};
type Member = {
  id: string;
  name: string;
  role: string;
  commune_id: string;
  email: string;
};
type Publication = {
  id: string;
  commune_id: string;
  title: string;
  body: string;
  category: string;
  visibility: "public" | "private";
  author_name: string;
  author_role: string;
  published_at: string;
  scope?: string;
  is_demo?: boolean;
  author_profile?: string;
  commune_name?: string;
  author_institution?: string;
};
type Modal = "login" | "create" | "about" | "members" | "register" | null;
const categories = [
  "Vie citoyenne",
  "Travaux",
  "Culture",
  "Environnement",
  "Services publics",
  "Conseil municipal",
];

const dateLabel = (date: string) =>
  new Intl.DateTimeFormat("fr-FR", { day: "numeric", month: "long" }).format(
    new Date(date),
  );
/** Forme française utilisée dans la signature, avec élision devant Annecy par exemple. */
function communeDesignation(name?: string) {
  if (!name) return "votre commune";
  return `la commune ${/^[aeiouyhàâéèêëîïôùûü]/i.test(name) ? "d’" : "de "}${name}`;
}
function CategoryIcon({
  category,
  size = 16,
}: {
  category: string;
  size?: number;
}) {
  if (category === "Environnement") return <Leaf size={size} />;
  if (category === "Culture") return <Sparkles size={size} />;
  if (category === "Travaux") return <Building2 size={size} />;
  if (category === "Services publics") return <Landmark size={size} />;
  return <Users size={size} />;
}

/** Illustration vectorielle locale : aucun service externe n'est nécessaire à l'affichage. */
function TownIllustration() {
  return (
    <svg className="town" viewBox="0 0 450 230" fill="none" aria-hidden="true">
      <circle cx="333" cy="57" r="30" fill="#f3c5cb" />
      <path
        d="M8 182C64 151 80 180 118 159S196 144 223 164s81-5 122-11 79 5 105 23v54H8Z"
        fill="#d6dff4"
      />
      <path d="M48 196h350" stroke="#8699c6" strokeWidth="2" />
      <path d="M263 116h94v80h-94z" fill="#ffffff" />
      <path d="m251 118 59-43 59 43Z" fill="#ed2939" />
      <path d="M279 136h15v23h-15zm43 0h15v23h-15z" fill="#6d85bd" />
      <path d="M302 166h19v30h-19z" fill="#6980b5" />
      <path d="M84 125h83v71H84z" fill="#ffffff" />
      <path d="m74 126 51-38 53 38Z" fill="#d95868" />
      <path d="M98 141h14v20H98zm33 0h14v20h-14z" fill="#6d85bd" />
      <path d="M108 176h20v20h-20z" fill="#6980b5" />
      <path d="M162 107h103v89H162z" fill="#ffffff" />
      <path d="m152 109 61-38 62 38Z" fill="#002395" />
      <path d="M201 41h25v46h-25z" fill="#ffffff" />
      <path d="m195 43 19-26 19 26Z" fill="#002395" />
      <circle cx="214" cy="61" r="8" fill="#edf1fc" stroke="#7b8cb3" />
      <path d="M214 55v7h4" stroke="#002395" />
      <path d="M181 126h16v24h-16zm48 0h16v24h-16z" fill="#6d85bd" />
      <path d="M203 168a11 11 0 0 1 22 0v28h-22z" fill="#7990c0" />
      <path d="M213 17V0" stroke="#002395" />
      <path d="M214 0h7v11h-7z" fill="#002395" />
      <path d="M221 0h7v11h-7z" fill="#fff" />
      <path d="M228 0h7v11h-7z" fill="#ed2939" />
      <path d="M48 195v-53m324 53v-53" stroke="#8c9fc8" strokeWidth="5" />
      <ellipse cx="48" cy="128" rx="23" ry="34" fill="#adbde1" />
      <ellipse cx="373" cy="129" rx="27" ry="37" fill="#8fa5d5" />
      <path d="M19 196h38m302 0h39" stroke="#8c9fc8" strokeWidth="3" />
      <path d="m196 230 16-34h11l24 34" fill="#e9edf7" />
    </svg>
  );
}

/** Dialogue accessible : focus contenu, fermeture par Échap et retour au bouton d'origine. */
function Dialog({
  title,
  close,
  children,
  wide = false,
}: {
  title: string;
  close: () => void;
  children: ReactNode;
  wide?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const closeRef = useRef(close);
  closeRef.current = close;
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    const oldOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    ref.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") closeRef.current();
      if (event.key === "Tab") {
        const items = ref.current?.querySelectorAll<HTMLElement>(
          'button, input, select, textarea, a[href], [tabindex="0"]',
        );
        if (!items?.length) return;
        const first = items[0],
          last = items[items.length - 1];
        if (
          event.shiftKey &&
          (document.activeElement === first ||
            document.activeElement === ref.current)
        ) {
          event.preventDefault();
          last.focus();
        } else if (
          !event.shiftKey &&
          (document.activeElement === last ||
            document.activeElement === ref.current)
        ) {
          event.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", onKey);
    return () => {
      document.body.style.overflow = oldOverflow;
      document.removeEventListener("keydown", onKey);
      previous?.focus();
    };
  }, []);
  return (
    <div
      className="modal-backdrop"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        ref={ref}
        className={`dialog ${wide ? "wide" : ""}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        tabIndex={-1}
      >
        <div className="dialog-head">
          <h2 id="dialog-title">{title}</h2>
          <button className="icon-button" onClick={close} aria-label="Fermer">
            <X size={21} />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

/** Réutilise la navigation municipale pour les pages publiques du Gouvernement. */
export default function Dashboard({
  children,
  section = "gouvernement",
}: {
  children?: ReactNode;
  section?: "gouvernement" | "presidence";
}) {
  const councilRequestHandled = useRef(false);
  const [communes, setCommunes] = useState<Commune[]>([]);
  const [catalog, setCatalog] = useState<CommuneCatalog | null>(null);
  const [member, setMember] = useState<Member | null>(null);
  const [publications, setPublications] = useState<Publication[]>([]);
  const [commune, setCommune] = useState("");
  const [category, setCategory] = useState("");
  const [query, setQuery] = useState("");
  const [search, setSearch] = useState("");
  const [view, setView] = useState<"public" | "council">("public");
  const [loading, setLoading] = useState(true);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState("");
  const [modal, setModal] = useState<Modal>(null);
  const [detail, setDetail] = useState<Publication | null>(null);
  const [discussionFocus, setDiscussionFocus] = useState<
    "comment" | "reaction" | undefined
  >();
  const [busy, setBusy] = useState(false);
  const [formError, setFormError] = useState("");
  const [toast, setToast] = useState("");
  const [revision, setRevision] = useState(0);
  const [menuOpen, setMenuOpen] = useState(false);
  const [demoLogin, setDemoLogin] = useState(true);
  const [loginCommune, setLoginCommune] = useState("annecy");
  const [loginRole, setLoginRole] = useState("maire");
  const [email, setEmail] = useState("maire.annecy@demo.qomune.fr");
  const [visibility, setVisibility] = useState<"public" | "private">("public");

  // Le choix effectué depuis l'annuaire ouvre le fil communal et survit au rechargement.
  useEffect(() => {
    if (children) return;
    const code = new URLSearchParams(window.location.search)
      .get("commune")
      ?.toUpperCase();
    if (code && /^[A-Z0-9]{5}$/.test(code)) setCommune(code);
  }, [children]);

  useEffect(() => {
    let current = true;
    Promise.all([
      api<Commune[]>("communes?limit=20"),
      api<CommuneCatalog>("communes/meta"),
      api<{ member: Member | null }>("auth/me"),
    ])
      .then(([items, catalog, session]) => {
        if (current) {
          setCommunes((previous) => [
            ...new Map([...previous, ...items].map((c) => [c.id, c])).values(),
          ]);
          setCatalog(catalog);
          setMember(session.member);
          setReady(true);
        }
      })
      .catch((e) => {
        if (current) {
          setError(e.message);
          setLoading(false);
        }
      });
    return () => {
      current = false;
    };
  }, [revision]);
  useEffect(() => {
    const timer = setTimeout(() => setSearch(query), 250);
    return () => clearTimeout(timer);
  }, [query]);
  useEffect(() => {
    if (!ready || children) return;
    let current = true;
    setLoading(true);
    setError("");
    setPublications([]);
    const params = new URLSearchParams({ scope: view });
    const selected = view === "council" ? member?.commune_id : commune;
    if (selected) params.set("commune", selected);
    if (category) params.set("category", category);
    if (search) params.set("q", search);
    api<Publication[]>(`publications?${params}`)
      .then((items) => {
        if (current) setPublications(items);
      })
      .catch((e) => {
        if (current) setError(e.message);
      })
      .finally(() => {
        if (current) setLoading(false);
      });
    return () => {
      current = false;
    };
  }, [ready, view, member, commune, category, search, revision, children]);
  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(""), 5000);
    return () => clearTimeout(timer);
  }, [toast]);

  // Le lien Conseil municipal des pages publiques conserve le parcours de connexion.
  useEffect(() => {
    if (!ready || children || councilRequestHandled.current) return;
    councilRequestHandled.current = true;
    if (
      new URLSearchParams(window.location.search).get("espace") === "conseil"
    ) {
      if (member) setView("council");
      else setModal("login");
    }
  }, [ready, member, children]);

  // Les noms des communes publiant dans le fil sont chargés par codes, indépendamment de la recherche.
  useEffect(() => {
    const ids = [
      ...new Set(
        [
          commune,
          member?.commune_id,
          ...publications.map((p) => p.commune_id),
        ].filter((id): id is string => Boolean(id)),
      ),
    ].filter((id) => !communes.some((c) => c.id === id));
    if (!ids.length) return;
    let current = true;
    api<Commune[]>(`communes?ids=${encodeURIComponent(ids.join(","))}`)
      .then((items) => {
        if (current)
          setCommunes((previous) => [
            ...new Map([...previous, ...items].map((c) => [c.id, c])).values(),
          ]);
      })
      .catch((error) => {
        if (current) setToast(error.message);
      });
    return () => {
      current = false;
    };
  }, [commune, member, publications, communes]);

  function selectCommune(selected: Commune | null) {
    if (children) {
      window.location.assign(
        selected ? `/?commune=${encodeURIComponent(selected.id)}` : "/",
      );
      return;
    }
    const url = new URL(window.location.href);
    if (selected) url.searchParams.set("commune", selected.id);
    else url.searchParams.delete("commune");
    window.history.replaceState(null, "", url);
    if (selected)
      setCommunes((previous) => [
        ...new Map([...previous, selected].map((c) => [c.id, c])).values(),
      ]);
    setCommune(selected?.id || "");
  }

  const selectedCommune = communes.find(
    (c) => c.id === (view === "council" ? member?.commune_id : commune),
  );
  const memberCommune = communes.find((c) => c.id === member?.commune_id);
  const featured = publications[0];
  const others = publications.slice(1);
  function openModal(next: Modal) {
    setFormError("");
    setModal(next);
    setMenuOpen(false);
  }
  function changeView(next: "public" | "council") {
    if (children) {
      window.location.assign(next === "council" ? "/?espace=conseil" : "/");
      return;
    }
    setMenuOpen(false);
    if (next === "council" && !member) {
      openModal("login");
      return;
    }
    setDetail(null);
    setPublications([]);
    setView(next);
    setCategory("");
    setQuery("");
  }
  async function login(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setFormError("");
    const data = new FormData(event.currentTarget);
    try {
      await api("auth/login", { email, password: data.get("password") });
      const session = await api<{ member: Member }>("auth/me");
      setMember(session.member);
      if (children) {
        window.location.assign("/?espace=conseil");
        return;
      }
      setView("council");
      setCategory("");
      setQuery("");
      setModal(null);
      setToast("Bienvenue dans votre espace municipal.");
    } catch (e) {
      setFormError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function logout() {
    setBusy(true);
    try {
      await api("auth/logout", {});
      setMember(null);
      setPublications([]);
      setView("public");
      setDetail(null);
      setToast("Vous êtes déconnecté.");
    } catch (e) {
      setToast((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function publish(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setFormError("");
    const data = new FormData(event.currentTarget);
    try {
      await api("publications", {
        title: data.get("title"),
        body: data.get("body"),
        category: data.get("category"),
        visibility,
      });
      setView("council");
      setCategory("");
      setQuery("");
      setModal(null);
      setRevision((n) => n + 1);
      setToast("Votre publication a bien été publiée.");
    } catch (e) {
      setFormError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }
  async function read(item: Publication, focus?: "comment" | "reaction") {
    try {
      setDiscussionFocus(focus);
      setDetail(await api<Publication>(`publications/${item.id}`));
    } catch (e) {
      setToast((e as Error).message);
    }
  }

  return (
    <div className="app-shell">
      <a className="skip-link" href="#main-content">
        Aller au contenu
      </a>
      <aside
        id="main-navigation"
        className={`sidebar ${menuOpen ? "is-open" : ""}`}
      >
        <a href="/" className="brand" aria-label="Qomune, accueil">
          <span className="brand-symbol">
            <Landmark size={23} />
          </span>
          qomune<span className="brand-dot">.</span>
        </a>
        <div className="sidebar-caption">LE LIEN LOCAL</div>
        <nav aria-label="Navigation principale">
          <button
            className={
              !children && view === "public" ? "nav-item active" : "nav-item"
            }
            onClick={() => changeView("public")}
          >
            <Globe2 size={19} /> Publications publiques{" "}
            <span className="nav-indicator" />
          </button>
          <button
            className={
              !children && view === "council" ? "nav-item active" : "nav-item"
            }
            onClick={() => changeView("council")}
          >
            <LockKeyhole size={18} /> Conseil municipal{" "}
            {!member && (
              <span className="small-lock">
                <LockKeyhole size={12} />
              </span>
            )}
          </button>
          <a
            href="/presidence"
            className={
              children && section === "presidence"
                ? "nav-item active"
                : "nav-item"
            }
            aria-current={
              children && section === "presidence" ? "page" : undefined
            }
          >
            <Flag size={19} />
            <span className="nav-label">Présidence de la République</span>
            {children && section === "presidence" && (
              <span className="nav-indicator" />
            )}
          </a>
          <a
            href="/gouvernement"
            className={
              children && section === "gouvernement"
                ? "nav-item active"
                : "nav-item"
            }
            aria-current={
              children && section === "gouvernement" ? "page" : undefined
            }
          >
            <Landmark size={19} /> Gouvernement
            {children && section === "gouvernement" && (
              <span className="nav-indicator" />
            )}
          </a>
        </nav>
        <div className="sidebar-line" />
        <p className="sidebar-label">VOTRE TERRITOIRE</p>
        <CommunePicker
          compact
          label="Choisir une commune"
          value={selectedCommune || null}
          onChange={selectCommune}
          disabled={!children && view === "council"}
        />
        <p className="territory-help">
          L’information locale commence
          <br />
          près de chez vous.
        </p>
        <div className="sidebar-bottom">
          <div className="council-invite">
            <span className="invite-icon">
              <Landmark size={22} />
            </span>
            <h3>Vous êtes élu ?</h3>
            <p>
              Faites vivre le dialogue
              <br />
              dans votre commune.
            </p>
            <button
              onClick={() =>
                member ? changeView("council") : openModal("login")
              }
            >
              Accéder à mon espace <ArrowUpRight size={15} />
            </button>
          </div>
          <button className="about-link" onClick={() => openModal("about")}>
            À propos de Qomune <ArrowUpRight size={14} />
          </button>
          <div className="sidebar-foot">
            <span className="status-dot" /> Prototype · qomune.fr
          </div>
        </div>
      </aside>
      {menuOpen && (
        <button
          className="menu-shade"
          aria-label="Fermer le menu"
          onClick={() => setMenuOpen(false)}
        />
      )}

      <div className="workspace">
        <header className="topbar">
          <div className="topbar-left">
            <button
              className="icon-button mobile-menu"
              aria-controls="main-navigation"
              aria-expanded={menuOpen}
              aria-label="Ouvrir le menu"
              onClick={() => setMenuOpen(!menuOpen)}
            >
              <Menu size={22} />
            </button>
            <span className="breadcrumb">L’espace des communes</span>
            <span className="prototype-badge">PROTOTYPE</span>
            {/* L’index public rassemble toutes les versions partageables du cahier des charges. */}
            <a
              className="specifications-link"
              href="https://github.com/medhi-tache/commune_gouv_fr/blob/main/docs/cahiers-des-charges/README.md"
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Consulter les cahiers des charges sur GitHub (nouvel onglet)"
            >
              Cahier des charges
            </a>
            <a
              className="specifications-link"
              href="https://github.com/medhi-tache/commune_gouv_fr"
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Consulter le dépôt GitHub (nouvel onglet)"
            >
              GitHub
            </a>
            <a
              className="specifications-link"
              href="https://www.linkedin.com/in/medhi-thomas-adrien-tache-1b46a2447/"
              target="_blank"
              rel="noopener noreferrer"
              aria-label="LinkedIn (nouvel onglet)"
            >
              LinkedIn
            </a>
            <a
              className="specifications-link"
              href="https://www.facebook.com/profile.php?id=61596023150910"
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Facebook (nouvel onglet)"
            >
              Facebook
            </a>
          </div>
          <div className="topbar-actions">
            <a className="specifications-link" href="/administration">
              Administration
            </a>
            {member ? (
              <>
                <span className="account-name">
                  <span className="avatar">
                    {member.name
                      .split(" ")
                      .map((n) => n[0])
                      .join("")}
                  </span>
                  <span>
                    {member.name}
                    <small>
                      {member.role === "maire"
                        ? "Maire"
                        : "Conseillère municipale"}{" "}
                      · {memberCommune?.name}
                    </small>
                  </span>
                </span>
                <button
                  className="icon-button"
                  onClick={logout}
                  disabled={busy}
                  aria-label="Se déconnecter"
                >
                  <LogOut size={18} />
                </button>
              </>
            ) : (
              <>
                <span className="visitor-label">
                  <span className="status-dot" /> Accès visiteur
                </span>
                <button
                  className="button login-button"
                  onClick={() => openModal("login")}
                >
                  <LockKeyhole size={15} /> Espace élus{" "}
                  <ArrowUpRight size={15} />
                </button>
              </>
            )}
          </div>
        </header>

        <main id="main-content">
          {children || (
            <>
              <section className="page-heading">
                <div>
                  <div className="eyebrow">
                    <span />{" "}
                    {view === "public"
                      ? "L’INFORMATION AU PLUS PRÈS DE VOUS"
                      : "VOTRE ESPACE DE TRAVAIL"}
                  </div>
                  <h1>
                    {view === "public"
                      ? "La vie locale, en lien direct."
                      : `Le conseil municipal${memberCommune ? ` ${/^[aeiouyhàâéèêëîïôùûü]/i.test(memberCommune.name) ? "d’" : "de "}${memberCommune.name}` : ""}.`}
                  </h1>
                  <p>
                    {view === "public"
                      ? "Les actualités de vos communes, partagées par celles et ceux qui les font vivre."
                      : "Informez les habitants et échangez avec les membres de votre conseil."}
                  </p>
                </div>
                {member && (
                  <div className="heading-actions">
                    {member.role === "maire" && (
                      <button
                        className="button"
                        onClick={() => openModal("members")}
                      >
                        <Users size={17} /> Membres du conseil
                      </button>
                    )}
                    <button
                      className="button primary"
                      onClick={() => {
                        setVisibility("public");
                        openModal("create");
                      }}
                    >
                      <Plus size={17} /> Publier
                    </button>
                  </div>
                )}
              </section>

              <section
                className={`welcome-banner ${view === "council" ? "private-banner" : ""}`}
              >
                <div className="welcome-copy">
                  <div className="banner-kicker">
                    {view === "public" ? (
                      <>
                        <Globe2 size={14} /> UN ESPACE PUBLIC, UN LIEN COMMUN
                      </>
                    ) : (
                      <>
                        <ShieldCheck size={15} /> UN ESPACE RÉSERVÉ À VOTRE
                        CONSEIL
                      </>
                    )}
                  </div>
                  <h2>
                    {view === "public" ? (
                      <>
                        Votre commune a des choses
                        <br />à vous partager.
                      </>
                    ) : (
                      <>
                        La bonne information.
                        <br />
                        Les bons destinataires.
                      </>
                    )}
                  </h2>
                  <p>
                    {view === "public"
                      ? "Projets, initiatives, rendez-vous… Retrouvez l’essentiel de la vie de votre territoire, en toute simplicité."
                      : "Chaque publication peut être publique ou réservée aux membres de votre conseil municipal."}
                  </p>
                  <div className="banner-note">
                    {view === "public" ? (
                      <>
                        <span className="tiny-flags">
                          <i />
                          <i />
                          <i />
                        </span>{" "}
                        Des communes et des citoyens, plus proches.
                      </>
                    ) : (
                      <>
                        <LockKeyhole size={14} /> L’accès privé est vérifié par
                        le serveur.
                      </>
                    )}
                  </div>
                </div>
                <TownIllustration />
              </section>

              <section className="feed-section" aria-labelledby="feed-title">
                <div className="section-title">
                  <div>
                    <h2 id="feed-title">
                      {view === "public"
                        ? "Le fil des communes"
                        : "Les publications du conseil"}
                      <span className="count-badge">
                        {loading ? "…" : publications.length}
                      </span>
                    </h2>
                    <p>
                      {selectedCommune
                        ? `${selectedCommune.name} · ${selectedCommune.department}`
                        : "Un regard sur ce qui se passe dans nos territoires."}
                    </p>
                  </div>
                  <span className="latest-label">
                    <ArrowDown size={14} /> Les plus récentes
                  </span>
                </div>

                <div className="filter-bar">
                  <label className="search-field">
                    <Search size={19} />
                    <input
                      aria-label="Rechercher une publication"
                      placeholder="Rechercher une publication…"
                      value={query}
                      maxLength={200}
                      onChange={(e) => setQuery(e.target.value)}
                    />
                    {query && (
                      <button
                        className="icon-button"
                        aria-label="Effacer la recherche"
                        onClick={() => setQuery("")}
                      >
                        <X size={15} />
                      </button>
                    )}
                  </label>
                  <CommunePicker
                    label="Filtrer par commune"
                    value={selectedCommune || null}
                    onChange={selectCommune}
                    disabled={view === "council"}
                  />
                  <div className="filter-select category-select">
                    <SlidersHorizontal size={17} />
                    <select
                      aria-label="Filtrer par thème"
                      value={category}
                      onChange={(e) => setCategory(e.target.value)}
                    >
                      <option value="">Tous les thèmes</option>
                      {categories.map((c) => (
                        <option key={c}>{c}</option>
                      ))}
                    </select>
                    <ChevronDown size={15} />
                  </div>
                </div>
                <div className="theme-tabs" aria-label="Thèmes">
                  <button
                    className={!category ? "selected" : ""}
                    onClick={() => setCategory("")}
                  >
                    Tout le fil
                  </button>
                  {categories
                    .filter(
                      (c) => c !== "Conseil municipal" || view === "council",
                    )
                    .map((c) => (
                      <button
                        key={c}
                        className={category === c ? "selected" : ""}
                        onClick={() => setCategory(c)}
                      >
                        <CategoryIcon category={c} />
                        {c}
                      </button>
                    ))}
                </div>

                {error ? (
                  <div role="alert" className="empty-state">
                    <ShieldCheck size={30} />
                    <h3>Le fil n’est pas disponible</h3>
                    <p>{error}</p>
                    <button
                      className="button"
                      onClick={() => setRevision((n) => n + 1)}
                    >
                      Réessayer
                    </button>
                  </div>
                ) : loading ? (
                  <div role="status" className="loading-state">
                    <span className="spinner" /> Chargement des publications…
                  </div>
                ) : !featured ? (
                  <div className="empty-state">
                    <MessageSquare size={30} />
                    <h3>Aucune publication pour le moment</h3>
                    <p>
                      {query || category || commune
                        ? "Essayez une autre recherche ou élargissez vos filtres."
                        : "Les publications de la commune apparaîtront ici."}
                    </p>
                    <button
                      className="button"
                      onClick={() => {
                        setQuery("");
                        setCategory("");
                        setCommune("");
                      }}
                    >
                      Réinitialiser les filtres
                    </button>
                  </div>
                ) : (
                  <>
                    <article className="featured-card">
                      <div
                        className={`featured-art art-${featured.category === "Travaux" ? "green" : featured.category === "Culture" ? "rose" : "blue"}`}
                      >
                        <span className="art-label">
                          <CategoryIcon
                            category={featured.category}
                            size={14}
                          />
                          {featured.category}
                        </span>
                        <TownIllustration />
                        <div className="art-caption">
                          <span>LA VIE DE NOS COMMUNES</span>
                          <strong>
                            {featured.scope === "national"
                              ? "Toutes les communes"
                              : communes.find(
                                  (c) => c.id === featured.commune_id,
                                )?.name}
                          </strong>
                        </div>
                      </div>
                      <div className="featured-body">
                        <PublicationMeta item={featured} communes={communes} />
                        <span className="featured-kicker">À LA UNE DU FIL</span>
                        <h3>
                          <button onClick={() => read(featured)}>
                            {featured.title}
                          </button>
                        </h3>
                        <p className="excerpt">
                          {publicationExcerpt(featured)}
                        </p>
                        <div className="card-bottom">
                          <Author item={featured} />
                          <button
                            className="read-button"
                            onClick={() => read(featured)}
                            aria-label={`Lire : ${featured.title}`}
                          >
                            <ArrowUpRight size={21} />
                          </button>
                        </div>
                      </div>
                      {view === "council" && (
                        <CouncilPostActions
                          title={featured.title}
                          onComment={() => read(featured, "comment")}
                          onReact={() => read(featured, "reaction")}
                        />
                      )}
                    </article>
                    <div className="publication-grid">
                      {others.map((item) => (
                        <article className="publication-card" key={item.id}>
                          <PublicationMeta item={item} communes={communes} />
                          <span
                            className={`category-pill pill-${item.category === "Culture" ? "rose" : item.category === "Environnement" ? "green" : "blue"}`}
                          >
                            <CategoryIcon category={item.category} size={13} />
                            {item.category}
                          </span>
                          <h3>
                            <button onClick={() => read(item)}>
                              {item.title}
                            </button>
                          </h3>
                          <p className="excerpt">{publicationExcerpt(item)}</p>
                          <div className="card-bottom">
                            <Author item={item} />
                            <button
                              className="read-button"
                              onClick={() => read(item)}
                              aria-label={`Lire : ${item.title}`}
                            >
                              <ArrowUpRight size={19} />
                            </button>
                          </div>
                          {view === "council" && (
                            <CouncilPostActions
                              title={item.title}
                              onComment={() => read(item, "comment")}
                              onReact={() => read(item, "reaction")}
                            />
                          )}
                        </article>
                      ))}
                    </div>
                    <div className="feed-end">
                      <span />
                      <span>Vous êtes à jour dans ce fil</span>
                      <span />
                    </div>
                  </>
                )}
              </section>
            </>
          )}
          <footer className="page-footer">
            <span>
              <Landmark size={15} /> Qomune · Le lien local
            </span>
            <span>
              {children ? (
                section === "presidence" ? (
                  "Présidence de la République · fiche publique"
                ) : (
                  "Gouvernement · annuaire public"
                )
              ) : (
                <>
                  {catalog?.total.toLocaleString("fr-FR") || "…"} communes · COG{" "}
                  {catalog?.vintage || "…"} · publications de démonstration
                </>
              )}
            </span>
            <button onClick={() => openModal("about")}>
              À propos <ArrowUpRight size={13} />
            </button>
          </footer>
        </main>
      </div>
      {toast && (
        <div className="toast" role="status">
          <Check size={18} />
          {toast}
          <button
            aria-label="Fermer la notification"
            onClick={() => setToast("")}
          >
            <X size={16} />
          </button>
        </div>
      )}

      {modal === "login" && (
        <Dialog
          title="Votre espace municipal"
          close={() => !busy && setModal(null)}
        >
          <p className="dialog-intro">
            Connectez-vous pour publier et retrouver les échanges de votre
            conseil municipal.
          </p>
          <div className="login-modes" aria-label="Mode de connexion">
            <button
              type="button"
              className={!demoLogin ? "selected" : ""}
              aria-pressed={!demoLogin}
              disabled={busy}
              onClick={() => {
                setDemoLogin(false);
                setEmail("");
                setFormError("");
              }}
            >
              Mon compte
            </button>
            <button
              type="button"
              className={demoLogin ? "selected" : ""}
              aria-pressed={demoLogin}
              disabled={busy}
              onClick={() => {
                setDemoLogin(true);
                setEmail(`${loginRole}.${loginCommune}@demo.qomune.fr`);
                setFormError("");
              }}
            >
              Démonstration
            </button>
          </div>
          {demoLogin ? (
            <div className="demo-notice">
              <ShieldCheck size={20} />
              <div>
                <strong>Connexion de démonstration</strong>
                <p>
                  Identités et mandats fictifs. Mot de passe :{" "}
                  <code>commune-demo</code>
                </p>
              </div>
            </div>
          ) : (
            <p className="dialog-intro">
              Utilisez les identifiants de votre compte de maire ou ceux remis
              par votre maire.
            </p>
          )}
          <form onSubmit={login} className="form">
            {demoLogin && (
              <div className="form-row">
                <label>
                  Commune de démonstration
                  <select
                    value={loginCommune}
                    onChange={(e) => {
                      setLoginCommune(e.target.value);
                      setLoginRole("maire");
                      setEmail(`maire.${e.target.value}@demo.qomune.fr`);
                    }}
                  >
                    {[
                      "annecy",
                      "bordeaux",
                      "lyon",
                      "nantes",
                      "saint-malo",
                      "strasbourg",
                    ].map((c) => (
                      <option key={c} value={c}>
                        {c === "saint-malo"
                          ? "Saint-Malo"
                          : c[0].toUpperCase() + c.slice(1)}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  Rôle
                  <select
                    value={loginRole}
                    onChange={(e) => {
                      setLoginRole(e.target.value);
                      setEmail(
                        `${e.target.value}.${loginCommune}@demo.qomune.fr`,
                      );
                    }}
                  >
                    <option value="maire">Maire</option>
                    {loginCommune === "annecy" && (
                      <option value="conseil">Conseillère municipale</option>
                    )}
                  </select>
                </label>
              </div>
            )}
            <label>
              Adresse e-mail
              <input
                type="email"
                autoComplete="username"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
              />
            </label>
            <label>
              Mot de passe
              <input
                type="password"
                name="password"
                autoComplete="current-password"
                key={demoLogin ? "demo-password" : "member-password"}
                defaultValue={demoLogin ? "commune-demo" : ""}
                required
                maxLength={200}
              />
            </label>
            {formError && (
              <p className="form-error" role="alert">
                {formError}
              </p>
            )}
            <button className="button primary full" disabled={busy}>
              {busy ? "Connexion…" : "Accéder à mon conseil municipal"}
              <ArrowRight size={17} />
            </button>
          </form>
          <button
            className="button full register-link"
            disabled={busy}
            onClick={() => openModal("register")}
          >
            Maire : créer mon compte
          </button>
          <p className="privacy-note">
            <LockKeyhole size={13} /> Seuls les membres de votre commune
            accèdent à son espace privé.
          </p>
        </Dialog>
      )}
      {modal === "register" && (
        <Dialog
          title="Inscription du maire"
          close={() => !busy && setModal(null)}
        >
          <MayorRegistration
            initialCommune={selectedCommune || null}
            onBusyChange={setBusy}
            onRegistered={async () => {
              const session = await api<{ member: Member }>("auth/me");
              setMember(session.member);
              setView("council");
              setCategory("");
              setQuery("");
              setModal(null);
              setToast(
                "Votre compte de maire est créé pour le prototype, sans vérification de mandat.",
              );
            }}
          />
        </Dialog>
      )}
      {modal === "members" && member?.role === "maire" && (
        <Dialog
          title="Membres du conseil"
          close={() => !busy && setModal(null)}
          wide
        >
          <CouncilMembers
            commune={memberCommune?.name || "votre commune"}
            onBusyChange={setBusy}
          />
        </Dialog>
      )}
      {modal === "create" && member && (
        <Dialog
          title="Nouvelle publication"
          close={() => !busy && setModal(null)}
          wide
        >
          <p className="dialog-intro">
            Publiez au nom de {communeDesignation(memberCommune?.name)}, avec
            votre nom comme auteur. Choisissez une publication publique ou
            réservée à votre conseil municipal.
          </p>
          <form onSubmit={publish} className="form">
            <fieldset className="visibility-choice">
              <legend>Visibilité de la publication</legend>
              <label className={visibility === "public" ? "chosen" : ""}>
                <input
                  type="radio"
                  name="visibility"
                  checked={visibility === "public"}
                  onChange={() => setVisibility("public")}
                />
                <Globe2 size={20} />
                <span>
                  <strong>Publique</strong>
                  <small>Tous les visiteurs</small>
                </span>
                {visibility === "public" && <Check size={16} />}
              </label>
              <label className={visibility === "private" ? "chosen" : ""}>
                <input
                  type="radio"
                  name="visibility"
                  checked={visibility === "private"}
                  onChange={() => setVisibility("private")}
                />
                <LockKeyhole size={20} />
                <span>
                  <strong>Privée</strong>
                  <small>Votre conseil uniquement</small>
                </span>
                {visibility === "private" && <Check size={16} />}
              </label>
            </fieldset>
            <label>
              Titre
              <input
                name="title"
                required
                minLength={3}
                maxLength={160}
                placeholder="Donnez un titre à votre publication"
              />
            </label>
            <label>
              Thème
              <select name="category" defaultValue="Vie citoyenne">
                {categories.map((c) => (
                  <option key={c}>{c}</option>
                ))}
              </select>
            </label>
            <label>
              Votre message
              <textarea
                name="body"
                required
                minLength={10}
                maxLength={10000}
                rows={7}
                placeholder="Partagez votre information…"
              />
            </label>
            <div className="publication-signature">
              <span className="avatar">
                {member.name
                  .split(" ")
                  .map((n) => n[0])
                  .join("")}
              </span>
              <span>
                Au nom de{" "}
                <strong>{communeDesignation(memberCommune?.name)}</strong>, par{" "}
                <strong>{member.name}</strong>
                <small>
                  {visibility === "public"
                    ? "Visible par tous"
                    : "Réservé au conseil municipal"}
                </small>
              </span>
            </div>
            {formError && (
              <p className="form-error" role="alert">
                {formError}
              </p>
            )}
            <div className="form-actions">
              <button
                className="button"
                type="button"
                disabled={busy}
                onClick={() => setModal(null)}
              >
                Annuler
              </button>
              <button className="button primary" disabled={busy}>
                {busy ? "Publication…" : "Publier le message"}
                <ArrowRight size={16} />
              </button>
            </div>
          </form>
        </Dialog>
      )}
      {detail && (
        <Dialog title="Publication" close={() => setDetail(null)} wide>
          <PublicationMeta item={detail} communes={communes} />
          <span className="category-pill">
            <CategoryIcon category={detail.category} />
            {detail.category}
          </span>
          <h3 className="detail-title">{detail.title}</h3>
          <p className="publication-attribution">
            {detail.is_demo && detail.author_profile ? (
              <>
                {detail.scope === "national"
                  ? "Publication nationale de démonstration"
                  : `Publication de démonstration destinée à ${detail.commune_name || detail.commune_id}`}
                , par <strong>{detail.author_name}</strong>.
              </>
            ) : (
              <>
                Publié au nom de{" "}
                <strong>
                  {communeDesignation(
                    communes.find((c) => c.id === detail.commune_id)?.name ||
                      detail.commune_id,
                  )}
                </strong>
                , par <strong>{detail.author_name}</strong>.
              </>
            )}
          </p>
          <Author item={detail} />
          {detail.visibility === "private" && (
            <div className="demo-notice">
              <LockKeyhole size={18} />
              <p>
                Réservé au conseil de{" "}
                {communeDesignation(
                  communes.find((c) => c.id === detail.commune_id)?.name,
                )}
                .
              </p>
            </div>
          )}
          <div className="detail-body">{detail.body}</div>
          {view === "council" && member?.commune_id === detail.commune_id && (
            <CouncilDiscussion
              key={detail.id}
              publicationId={detail.id}
              focus={discussionFocus}
            />
          )}

          <button className="text-button" onClick={() => setDetail(null)}>
            <ArrowLeft size={16} /> Revenir au fil
          </button>
        </Dialog>
      )}
      {modal === "about" && (
        <Dialog
          title="Le lien local, tout simplement."
          close={() => setModal(null)}
        >
          <p className="dialog-intro">
            Qomune permet aux maires et aux conseillers municipaux de partager
            la vie de leur commune avec les habitants et de publier dans un
            espace réservé à leur conseil.
          </p>
          <div className="about-block">
            <Globe2 />
            <div>
              <h3>Un fil ouvert à tous</h3>
              <p>Les publications publiques sont consultables sans compte.</p>
            </div>
          </div>
          <div className="about-block">
            <LockKeyhole />
            <div>
              <h3>Un espace pour chaque conseil</h3>
              <p>
                Les publications privées sont limitées aux membres de la commune
                concernée.
              </p>
            </div>
          </div>
          <div className="demo-notice">
            <Landmark size={22} />
            <div>
              <strong>Un prototype en construction</strong>
              <p>
                {catalog?.total.toLocaleString("fr-FR") || "…"} communes du
                référentiel Insee COG {catalog?.vintage || "…"}, téléchargé
                depuis data.gouv.fr. Les comptes et les publications de
                démonstration sont fictifs. Qomune n’est pas un service officiel
                de l’État. Le périmètre cible du projet est de 34 875 communes.
              </p>
            </div>
          </div>
          <p className="privacy-note">
            Prototype prévu : qomune.fr. Domaine définitif envisagé :
            commune.gouv.fr, sous réserve de son attribution.
          </p>
        </Dialog>
      )}
    </div>
  );
}

function PublicationMeta({
  item,
  communes,
}: {
  item: Publication;
  communes: Commune[];
}) {
  const commune = communes.find((c) => c.id === item.commune_id);
  return (
    <div className="publication-meta">
      <span className="commune-emblem">
        <Landmark size={14} />
      </span>
      <span className="commune-name">
        {item.scope === "national"
          ? "Toutes les communes"
          : commune?.name || item.commune_id}
      </span>
      <span className="meta-separator">·</span>
      <time dateTime={item.published_at}>{dateLabel(item.published_at)}</time>
      {item.is_demo && item.author_profile && (
        <span className="national-demo-tag">
          {item.author_institution === "presidence"
            ? item.scope === "national"
              ? "Présidence · National"
              : "Présidence"
            : item.scope === "national"
              ? "National"
              : "Gouvernement"}{" "}
          · Démonstration
        </span>
      )}
      {item.visibility === "private" && (
        <span className="private-tag">
          <LockKeyhole size={12} /> Privé
        </span>
      )}
    </div>
  );
}
/** Le marquage fictif reste dans le titre et le badge ; l'extrait montre le message. */
function publicationExcerpt(item: Publication) {
  const content =
    item.is_demo && item.author_profile
      ? item.body.split("\n\n").slice(1).join("\n\n")
      : item.body;
  return content.split("\n")[0];
}
function Author({ item }: { item: Publication }) {
  return (
    <div className="author">
      <span className="author-avatar">
        {item.author_name
          .split(" ")
          .slice(0, 2)
          .map((n) => n[0])
          .join("")}
      </span>
      <span>
        {item.author_profile ? (
          <a href={item.author_profile}>{item.author_name}</a>
        ) : (
          item.author_name
        )}
        <small>
          {item.is_demo && item.author_profile
            ? item.author_role
            : item.author_role === "maire"
              ? "Maire"
              : "Membre du conseil municipal"}
        </small>
      </span>
    </div>
  );
}

/** Ces raccourcis n'apparaissent que dans le fil authentifié du conseil. */
function CouncilPostActions({
  title,
  onComment,
  onReact,
}: {
  title: string;
  onComment: () => void;
  onReact: () => void;
}) {
  return (
    <div className="council-post-actions">
      <button type="button" aria-label={`Réagir : ${title}`} onClick={onReact}>
        <ThumbsUp size={15} /> Réagir
      </button>
      <button
        type="button"
        aria-label={`Commenter : ${title}`}
        onClick={onComment}
      >
        <MessageSquare size={15} /> Commenter
      </button>
    </div>
  );
}
