# commune_gouv_fr

Projet de diffusion de publications pour les 34 875 communes de France.

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

- **Backend : Rust** — API, authentification, gestion des publications et contrôle des accès par commune et par visibilité.
- **Frontend : Next.js** — consultation des publications publiques et espace connecté des maires et des conseillers municipaux.

Le frontend communique avec le backend via une API. Le backend vérifie les droits à chaque accès aux publications privées et à leurs pièces jointes.

## État du projet

Le dépôt est initialisé. Les rôles, les règles de visibilité et les technologies principales sont définis. L'application reste à développer. Les modalités de vérification des mandats des élus, les formats de publication et les canaux de diffusion restent à préciser.
