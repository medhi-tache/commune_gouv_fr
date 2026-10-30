# commune_gouv_fr — prototype Qomune

Projet de diffusion de publications pour les 34 875 communes de France.

**Stack : Rust · MongoDB · Next.js**

## Objectif

Permettre aux maires et aux conseillers municipaux de chaque commune de publier des informations publiques pour les visiteurs et des informations privées pour leur conseil municipal.

## Périmètre cible

Le prototype charge les **34 875 communes actuelles** de France métropolitaine et des DROM du **Code officiel géographique 2026 de l’Insee**, référencé sur [data.gouv.fr](https://www.data.gouv.fr/datasets/code-officiel-geographique-1). Les communes déléguées, les communes associées et les arrondissements municipaux sont exclus (`TYPECOM=COM`).

## Utilisateurs et publications

| Utilisateur | Accès et actions |
| --- | --- |
| Visiteur | Consulter les publications publiques des communes. |
| Maire | Publier des informations publiques ou privées, consulter les publications privées et ajouter les membres de son conseil municipal. |
| Conseiller municipal | Publier des informations publiques ou privées pour sa commune et consulter les publications privées de son conseil municipal. |

## Visibilité des publications

Chaque publication possède un mode de visibilité :

- **Public** : accessible à tous les visiteurs, sans connexion.
- **Privé** : accessible uniquement aux membres authentifiés du conseil municipal de la commune concernée.

Chaque membre du conseil, maire ou conseiller municipal, peut publier en choisissant **Public** ou **Privé**. Chaque message est publié **au nom de sa commune et signé du nom de son auteur**. La commune, le nom et le rôle de l'auteur sont déterminés par sa session côté serveur ; ils ne sont pas modifiables dans le formulaire. La signature est affichée avant l'envoi, dans le fil et dans la lecture détaillée.

Les publications privées ne doivent être accessibles ni aux visiteurs ni aux membres des conseils municipaux des autres communes. Le contrôle d'accès doit s'appliquer au contenu et aux éventuelles pièces jointes, y compris lors d'un accès par lien direct.

## Architecture technique

- **Backend : Rust / Axum** — API, authentification, gestion des publications et contrôle des accès par commune et par visibilité.
- **Frontend : Next.js / React / TypeScript** — consultation des publications publiques et espace connecté des maires et des conseillers municipaux.
- **Base de données : MongoDB** — conservation des publications, des comptes de démonstration et des sessions ; index de recherche et expiration des sessions.

L'identité visuelle suit une palette **bleu, blanc, rouge** : bleu `#002395` pour la navigation et les actions, blanc `#FFFFFF` pour les surfaces, rouge `#ED2939` pour les accents.

Le frontend communique avec le backend via une API. Le backend vérifie les droits à chaque accès aux publications privées et à leurs pièces jointes.

## Domaines prévus

| Version | Adresse |
| --- | --- |
| Prototype | https://qomune.fr |
| Version définitive | https://commune.gouv.fr |

Ces adresses correspondent aux destinations prévues du projet. Leur mention ne confirme ni leur configuration ni le déploiement de l'application.

## Lancer le prototype

Prérequis : Node.js 20.9 ou plus récent avec npm, et Rust 1.88 ou plus récent avec Cargo, et MongoDB 8 local ou via Docker Compose. Une connexion réseau est nécessaire lors de la première installation des dépendances. Les versions résolues sont enregistrées dans `frontend/package-lock.json` et `backend/Cargo.lock`.

Depuis la racine du dépôt :

```sh
npm run setup
docker compose up -d mongodb
npm run dev
```

Si MongoDB est déjà démarré sur `127.0.0.1:27017`, la commande Docker est inutile. Pour une autre instance : `MONGODB_URI=mongodb://127.0.0.1:27018 npm run dev`.

Ouvrir **http://127.0.0.1:3000**. Le backend écoute sur **http://127.0.0.1:8080**. Le script attend la fin de l’import communal et la disponibilité de l’API avant de démarrer Next.js. `Ctrl+C` arrête les deux services.

Si les ports sont déjà utilisés :

```sh
WEB_PORT=3100 API_BIND=127.0.0.1:8180 npm run dev
```

Le script transmet automatiquement l'adresse du backend à Next.js. Les fichiers `.env.example` décrivent les variables disponibles ; le script de lancement ne les charge pas automatiquement. Exporter les variables dans le terminal pour un lancement commun. Next.js charge aussi son propre fichier `frontend/.env.local` lorsqu'il est lancé séparément.

## Essayer les parcours

1. Consulter le fil public sans connexion. Filtrer par commune, thème ou mot recherché, puis ouvrir une publication.
2. Cliquer sur **Espace élus**. Le formulaire permet de choisir une commune et un compte de démonstration.
3. Se connecter avec l'un des comptes ci-dessous. Le mot de passe initial commun est **`commune-demo`**.
4. Cliquer sur **Publier**, choisir **Publique** ou **Privée**, puis remplir et envoyer le formulaire.
5. Se déconnecter : seules les publications publiques restent visibles. Un autre conseil municipal n'accède pas aux publications privées de cette commune.

| Compte | Rôle et commune |
| --- | --- |
| `maire.annecy@demo.qomune.fr` | Maire d'Annecy |
| `conseil.annecy@demo.qomune.fr` | Conseillère municipale d'Annecy |
| `maire.bordeaux@demo.qomune.fr` | Maire de Bordeaux |
| `maire.lyon@demo.qomune.fr` | Maire de Lyon |
| `maire.nantes@demo.qomune.fr` | Maire de Nantes |
| `maire.saint-malo@demo.qomune.fr` | Maire de Saint-Malo |
| `maire.strasbourg@demo.qomune.fr` | Maire de Strasbourg |

Les noms des élus, les mandats et les contenus sont fictifs. Ce prototype n'est pas un service officiel de l'État.

## Commentaires et réactions dans l'espace sécurisé

Dans **Conseil municipal**, chaque publication dispose des boutons **Commenter** et **Réagir**, qui ouvrent ses échanges internes. Les maires et conseillers de la commune peuvent :

- commenter sous leur nom, avec leur fonction et la date du message ;
- choisir une réaction : 👍 J’aime, ❤️ J’adore, 😆 Haha, 😮 Waouh, 😢 Triste ou 😡 Grrr ;
- changer leur réaction ou cliquer de nouveau sur la réaction sélectionnée pour la retirer ;
- consulter les compteurs, les noms des membres ayant réagi et les commentaires précédents.

Une seule réaction est conservée par membre et publication. Les commentaires contiennent de 1 à 2 000 caractères. Les 50 plus récents sont chargés à l'ouverture, puis le bouton **Voir les commentaires précédents** permet de remonter le fil. **Actualiser les échanges** recharge les contributions des autres membres ; il n'y a pas de mise à jour en temps réel dans ce prototype.

**Ces échanges restent privés au conseil, même lorsque la publication principale est publique.** Les visiteurs et les membres d'une autre commune ne peuvent consulter ni les commentaires, ni les réactions, ni leurs compteurs. Les auteurs et la commune sont toujours déterminés par le serveur. Les données sont conservées dans MongoDB après un redémarrage.

La modification et la suppression des commentaires ainsi que leur modération restent à développer.

## Référentiel des communes : data.gouv.fr / Insee

Les fichiers du COG 2026 sont découverts dans les métadonnées du jeu de données officiel data.gouv.fr puis téléchargés depuis les URL de l'Insee qui y sont publiées. Les fichiers des départements et régions complètent le libellé de chaque commune.

- 34 875 communes actuelles ; codes INSEE conservés comme chaînes, y compris les zéros initiaux et les codes corses `2A` et `2B`.
- Source et licence : **Insee — Licence Ouverte / Open Licence**, selon [la fiche data.gouv.fr](https://www.data.gouv.fr/datasets/code-officiel-geographique-1).
- Données normalisées : `backend/reference/communes-2026.json` ; ressources et empreintes SHA-256 : `backend/reference/source.json`.
- Import automatique au premier démarrage, puis vérification du marqueur et du nombre de documents aux démarrages suivants. Aucun téléchargement réseau n'est nécessaire au démarrage : le référentiel validé est embarqué dans le backend.
- Recherche par nom ou code INSEE, insensible à la casse, aux accents, aux apostrophes et aux tirets. Le sélecteur affiche au plus 20 résultats par recherche ; il faut préciser la saisie pour trouver les autres.

Pour retélécharger et réimporter le millésime 2026 (Python 3 standard, connexion Internet et MongoDB nécessaires) :

```sh
npm run communes:refresh
```

Pour réimporter les fichiers déjà présents sans téléchargement :

```sh
cargo run --manifest-path backend/Cargo.toml -- --import-communes
```

Exporter `MONGODB_URI` si la base utilise un autre port. La mise à jour porte sur les documents des communes, sans modifier les comptes, les sessions ni les publications. Une opération interrompue est reprise au prochain lancement. Le script reste explicitement fixé au millésime 2026 ; un changement de millésime nécessite une mise à jour des ressources et une vérification du périmètre.

## Inscription du maire : prototype

L'inscription du maire est **sans vérification dans le prototype**. Ouvrir **Espace élus → Maire : créer mon compte**, choisir sa commune, puis renseigner son nom, son adresse e-mail et un mot de passe d'au moins 12 caractères. Le compte est immédiatement utilisable et reste marqué comme non vérifié. Un seul maire peut s'inscrire par commune ; les six communes de démonstration ont déjà leurs comptes préconfigurés.

En production, le parcours prévu combinera **FranceConnect pour l'identité** et le **Répertoire national des élus (RNE), disponible sur data.gouv.fr, pour le mandat de maire dans la commune**. Cette intégration n'est pas encore implémentée. Le mode `QOMUNE_MODE=production` refuse donc de démarrer le serveur actuel ; aucune connexion FranceConnect n'est simulée.

Les sources officielles et le parcours cible sont détaillés dans [AUTHENTIFICATION.md](AUTHENTIFICATION.md).

## Ajouter les membres du conseil

1. Se connecter avec un compte de maire, puis ouvrir **Membres du conseil**.
2. Saisir le nom complet et l'adresse e-mail du membre, puis cliquer sur **Créer le compte du membre**.
3. Noter ou copier les identifiants affichés : le mot de passe initial est généré automatiquement et affiché uniquement à la création. Seul son hash Argon2 est conservé dans MongoDB.
4. Transmettre les identifiants au membre par un moyen privé. Aucun e-mail n'est envoyé par le prototype.
5. Le membre ouvre **Espace élus → Mon compte** et se connecte avec ces identifiants. Il peut consulter les publications privées de son conseil et publier pour sa commune.

Le nouveau compte reçoit obligatoirement le rôle de conseiller municipal dans la commune du maire connecté. Le serveur refuse les ajouts par un visiteur ou un conseiller, les tentatives de choisir une autre commune ou le rôle de maire, ainsi que les adresses déjà utilisées. Un doublon ne modifie jamais un compte existant. La liste des membres et leurs adresses e-mail restent réservées au maire de cette commune.

## Données et configuration

Le backend utilise la base MongoDB `qomune_prototype`, avec les collections `communes`, `members`, `sessions`, `publications`, `comments`, `reactions` et `reference_imports` (traçabilité des imports). Le fichier `compose.yaml` fournit une instance locale avec un volume persistant et un port accessible uniquement sur la machine. Les publications restent présentes après un redémarrage.

Les comptes et publications de démonstration sont insérés avec `$setOnInsert` : une nouvelle initialisation ne remplace ni les comptes ni les publications existantes. Pour essayer un jeu de données neuf sans effacer l'ancien, lancer avec un autre nom de base : `MONGODB_DATABASE=qomune_autre_demo npm run dev`.

MongoDB doit rester un service interne. La configuration Docker locale ne prévoit pas d'authentification et ne doit pas être exposée sur Internet.

| Variable | Valeur par défaut | Usage |
| --- | --- | --- |
| `QOMUNE_MODE` | `prototype` | Inscription déclarative autorisée uniquement dans ce mode. La production attend FranceConnect + contrôle du mandat RNE. |
| `WEB_PORT` | `3000` | Port du frontend via le script commun. |
| `API_BIND` | `127.0.0.1:8080` | Adresse d'écoute Rust. |
| `API_BASE_URL` | `http://127.0.0.1:8080` | Adresse interne Rust pour Next.js ; adaptée par le script commun si `API_BIND` change. |
| `APP_ORIGIN` | Origine HTTP locale calculée depuis la requête | Origine publique exacte à autoriser derrière un proxy, par exemple `https://qomune.fr`. |
| `MONGODB_URI` | `mongodb://127.0.0.1:27017` | Adresse interne de MongoDB ; peut contenir des identifiants à conserver hors de Git. |
| `MONGODB_DATABASE` | `qomune_prototype` | Nom de la base de l’application. |
| `QOMUNE_DEMO_PASSWORD` | `commune-demo` | Mot de passe de création des comptes, uniquement à la première initialisation de la base. |
| `COOKIE_SECURE` | `false` | Activer avec `true` sur le backend derrière HTTPS. |

Les mots de passe sont hachés avec Argon2. Les sessions utilisent un cookie `HttpOnly`, `SameSite=Strict`, expirent au bout de huit heures et sont révoquées à la déconnexion. Le serveur Rust reste lié à l'interface locale par défaut ; le navigateur utilise le relais Next.js sur la même origine.

## Vérification et documentation

```sh
npm run check
```

MongoDB doit être démarré avant cette commande. Elle vérifie le format Rust, lance les vingt-deux tests, dont vingt et un tests d’intégration contre MongoDB et un test d’intégrité du référentiel et construit le frontend avec vérification TypeScript. Chaque test crée une base temporaire `qomune_test_<UUID>` puis la supprime, sans toucher à la base de l’application. La variable `MONGODB_URI` permet aussi de choisir l’instance pour les tests.

- [Guide du code et règles d'accès](CONTRIBUTING.md)
- [Contrat de l'API](API.md)
- [Inscription et vérification des maires : FranceConnect + RNE](AUTHENTIFICATION.md)

Le code, ses commentaires explicatifs et la documentation du projet sont en français.

## État et limites du prototype

Le prototype implémente le fil public, les filtres, la lecture détaillée, la connexion, l'espace du conseil, l'inscription déclarative des maires, l'ajout de conseillers par le maire et la création de publications textuelles publiques ou privées, ainsi que leurs commentaires et réactions internes au conseil. Le référentiel comporte les 34 875 communes du COG 2026. Les comptes préconfigurés et les publications d'exemple restent limités à six communes ; aucun compte de maire n'est créé automatiquement pour les autres communes.

La vérification réelle des mandats, les invitations par e-mail, la récupération et le changement de mot de passe, la modification et la suppression des membres, les pièces jointes, la modification et la suppression des publications, les notifications et la modération restent à développer. Les requêtes du fil sont limitées aux 100 dernières publications correspondant aux filtres, sans pagination. La recherche est une recherche littérale par sous-chaîne, sans interprétation des opérateurs de regex.

Le prototype fonctionne localement et n'est pas déployé sur les domaines prévus. Pour une mise en ligne réelle sur qomune.fr, prévoir un hébergement exécutant Rust et Next.js, une instance MongoDB privée avec stockage persistant, HTTPS, les cookies sécurisés et une configuration de proxy préservant l'origine des requêtes. Il faudra aussi remplacer les comptes de démonstration par des comptes vérifiés, ajouter une limitation des tentatives de connexion et organiser les sauvegardes avant d'accueillir des données réelles.

Le domaine définitif envisagé `commune.gouv.fr` reste soumis à son attribution et à sa configuration.

## Administration et graphiques d’utilisation

L’espace **Administration** est accessible depuis l’en-tête et à l’adresse `/administration`. Il dispose d’une authentification indépendante de celle des élus. Un maire ou un conseiller municipal ne peut pas consulter les statistiques globales avec sa session municipale.

Avec `npm run dev`, un mot de passe aléatoire est créé à la première utilisation dans `backend/data/administration-password.txt` (fichier local ignoré par Git, permissions limitées au propriétaire). Saisir ce mot de passe dans l’espace Administration. Il n’est jamais affiché dans les journaux ni intégré au frontend. Pour un lancement direct de Rust ou un hébergement distant, fournir `QOMUNE_ADMIN_PASSWORD` (16 à 200 caractères) via la configuration privée du serveur. Sa modification révoque les sessions administrateur existantes. Ne pas publier ce secret. Le compte configuré est conservé dans MongoDB sous forme de hash Argon2 ; retirer la variable ne supprime pas ce compte.

Le tableau de bord présente :

- les communes disposant d’au moins un compte municipal, les membres et les maires ;
- les volumes actuels de publications publiques et privées, de commentaires et de réactions ;
- les publications quotidiennes, les thèmes et les cinq communes les plus actives sur 7, 30 ou 90 jours calendaires en UTC, aujourd’hui inclus ;
- une répartition globale publique/privée et un tableau quotidien accessible en complément du graphique.

Les chiffres proviennent de MongoDB et incluent les données de démonstration. Aucune visite, adresse IP, durée de consultation ou identité de visiteur n’est collectée pour ces graphiques. Les graphiques ne révèlent ni les textes privés ni les coordonnées des élus. Les compteurs globaux décrivent l’état actuel de la base ; seuls les graphiques d’activité sont filtrés par période. Il n’y a pas encore de gestion des communes ou des comptes depuis cette première page de pilotage.

Les routes `POST /admin/login`, `POST /admin/logout` et `GET /admin/statistics?days=30` passent par le relais `/api`. La session administrateur expire après deux heures et utilise un cookie HttpOnly, SameSite=Strict (Secure lorsque `COOKIE_SECURE=true`). Les résultats ne sont pas mis en cache. Les tests vérifient l’isolation des sessions, les périodes, les journées sans activité et la révocation des accès.

## Hébergement OVHcloud

La procédure et les fichiers de déploiement du prototype sont décrits dans [deploy/README.md](deploy/README.md). La configuration distante utilise `compose.ovh.yaml`, distinct de la base de développement locale.
