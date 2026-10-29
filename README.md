# commune_gouv_fr — prototype Qomune

Projet de diffusion de publications pour les 34 875 communes de France.

**Stack : Rust · MongoDB · Next.js**

## Objectif

Permettre aux maires et aux conseillers municipaux de chaque commune de publier des informations publiques pour les visiteurs et des informations privées pour leur conseil municipal.

## Périmètre cible

Le projet vise les 34 875 communes de France, selon le périmètre défini à son lancement.

## Utilisateurs et publications

| Utilisateur | Accès et actions |
| --- | --- |
| Visiteur | Consulter les publications publiques des communes. |
| Maire | Publier des informations publiques ou privées pour sa commune et consulter les publications privées de son conseil municipal. |
| Conseiller municipal | Publier des informations publiques ou privées pour sa commune et consulter les publications privées de son conseil municipal. |

## Visibilité des publications

Chaque publication possède un mode de visibilité :

- **Public** : accessible à tous les visiteurs, sans connexion.
- **Privé** : accessible uniquement aux membres authentifiés du conseil municipal de la commune concernée.

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

Ouvrir **http://127.0.0.1:3000**. Le backend écoute sur **http://127.0.0.1:8080**. `Ctrl+C` arrête les deux services.

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

## Données et configuration

Le backend utilise la base MongoDB `qomune_prototype`, avec les collections `communes`, `members`, `sessions` et `publications`. Le fichier `compose.yaml` fournit une instance locale avec un volume persistant et un port accessible uniquement sur la machine. Les publications restent présentes après un redémarrage.

Les exemples sont insérés avec `$setOnInsert` : une nouvelle initialisation ne remplace ni les comptes ni les publications existantes. Pour essayer un jeu de données neuf sans effacer l'ancien, lancer avec un autre nom de base : `MONGODB_DATABASE=qomune_autre_demo npm run dev`.

MongoDB doit rester un service interne. La configuration Docker locale ne prévoit pas d'authentification et ne doit pas être exposée sur Internet.

| Variable | Valeur par défaut | Usage |
| --- | --- | --- |
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

MongoDB doit être démarré avant cette commande. Elle vérifie le format Rust, lance les six tests d’intégration contre MongoDB et construit le frontend avec vérification TypeScript. Chaque test crée une base temporaire `qomune_test_<UUID>` puis la supprime, sans toucher à la base de l’application. La variable `MONGODB_URI` permet aussi de choisir l’instance pour les tests.

- [Guide du code et règles d'accès](CONTRIBUTING.md)
- [Contrat de l'API](API.md)

Le code, ses commentaires explicatifs et la documentation du projet sont en français.

## État et limites du prototype

Le prototype implémente le fil public, les filtres, la lecture détaillée, la connexion de démonstration, l'espace du conseil et la création de publications textuelles publiques ou privées. Il comprend six communes d'exemple ; le référentiel complet des 34 875 communes n'est pas importé.

La vérification réelle des mandats, les invitations de membres, la récupération de mot de passe, les pièces jointes, la modification et la suppression des publications, les notifications et la modération restent à développer. Les requêtes du fil sont limitées aux 100 dernières publications correspondant aux filtres, sans pagination. La recherche est une recherche littérale par sous-chaîne, sans interprétation des opérateurs de regex.

Le prototype fonctionne localement et n'est pas déployé sur les domaines prévus. Pour une mise en ligne réelle sur qomune.fr, prévoir un hébergement exécutant Rust et Next.js, une instance MongoDB privée avec stockage persistant, HTTPS, les cookies sécurisés et une configuration de proxy préservant l'origine des requêtes. Il faudra aussi remplacer les comptes de démonstration par des comptes vérifiés, ajouter une limitation des tentatives de connexion et organiser les sauvegardes avant d'accueillir des données réelles.

Le domaine définitif envisagé `commune.gouv.fr` reste soumis à son attribution et à sa configuration.
