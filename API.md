# API du prototype

Préfixe navigateur : `/api`. Le serveur Rust expose les mêmes chemins sans ce préfixe sur `127.0.0.1:8080`. Les réponses contiennent du JSON. Les données du relais Next.js ne sont pas mises en cache.

## Routes

| Méthode | Chemin | Comportement |
| --- | --- | --- |
| GET | `/health` | État du serveur. |
| GET | `/communes` | Les six communes de démonstration. |
| POST | `/auth/login` | Connexion par `email` et `password` ; pose un cookie de session. |
| POST | `/auth/logout` | Révoque la session et efface son cookie. Corps : `{}`. |
| GET | `/auth/me` | `{ "member": ... }` ou `{ "member": null }`. |
| GET | `/members` | Liste des membres de sa commune, réservée au maire. |
| POST | `/members` | Création d’un conseiller municipal par le maire de la commune. |
| GET | `/publications` | Fil public par défaut ; au maximum 100 résultats, du plus récent au plus ancien. |
| GET | `/publications/{id}` | Publication publique, ou publication privée de sa propre commune. |
| POST | `/publications` | Création par un maire ou conseiller connecté. |

## Fil et filtres

Paramètres facultatifs :

- `scope=public` : uniquement les publications publiques ; valeur par défaut.
- `scope=council` : publications publiques et privées de sa commune ; connexion obligatoire.
- `commune=74010` : filtre par code de commune. Avec `scope=council`, une autre commune est refusée.
- `category=Culture` : filtre exact par thème.
- `q=...` : recherche dans le titre et le texte, limitée à 200 octets UTF-8.

## Créer une publication

```json
{
  "title": "Notre prochaine rencontre citoyenne",
  "body": "La commune vous invite à une rencontre ouverte à toutes et tous.",
  "category": "Vie citoyenne",
  "visibility": "public"
}
```

Le titre contient 3 à 160 caractères, et le texte 10 à 10 000 caractères après suppression des espaces aux extrémités. Les visibilités acceptées sont `public` et `private`. Les thèmes sont `Vie citoyenne`, `Travaux`, `Culture`, `Environnement`, `Services publics` et `Conseil municipal`.

Les maires et les conseillers peuvent publier avec les deux visibilités. L'identité de l'auteur, son rôle et sa commune sont déterminés par la session. Les lectures renvoient `commune_id`, `author_name` et `author_role` pour afficher une publication au nom de la commune et de la personne qui l’a écrite. Les champs supplémentaires sont refusés. Réponse : `201` avec `{ "id": "..." }`.

Le relais Next.js exige une origine identique à celle de la requête ainsi qu'un contenu `application/json` pour les écritures. Il n'accepte que les routes énumérées ci-dessus. Taille maximale d'une requête : 64 Kio.

## Créer les comptes des membres du conseil

`POST /members` exige une session de maire et accepte uniquement :

```json
{
  "name": "Alex Dupont",
  "email": "alex.dupont@exemple.fr"
}
```

Le nom contient de 2 à 120 caractères après suppression des espaces aux extrémités et aucun caractère de contrôle. L'adresse e-mail usuelle ASCII contient au plus 200 caractères, est normalisée en minuscules et doit être unique dans l'ensemble de l'application.

Le serveur impose le rôle `conseiller` et la commune du maire connecté. Les champs supplémentaires tels que `role`, `commune_id` et `password` sont refusés. La réponse `201` contient `{ "member": { ... }, "initial_password": "..." }`. Le mot de passe aléatoire est retourné uniquement à la création, puis stocké sous forme de hash Argon2. Le champ interne `created_by` conserve l'identifiant du maire ayant créé le compte.

`GET /members` renvoie uniquement les profils publics de stockage (`id`, `name`, `email`, `role`, `commune_id`) de la commune du maire connecté, sans hash ni mot de passe initial. Ces coordonnées ne sont pas consultables par les visiteurs ou les conseillers. Aucun paramètre ne permet de choisir une autre commune.

Un doublon d'adresse renvoie `409`, y compris pour des créations concurrentes. Aucun compte existant n'est modifié. Le nouveau membre peut immédiatement se connecter avec `/auth/login` puis utiliser les routes de publication selon les droits de sa commune. Aucun e-mail n'est envoyé.

## Erreurs

Les erreurs métier prennent la forme `{ "error": "Message en français" }`.

- `400` : contenu ou filtre invalide.
- `401` : connexion requise ou identifiants incorrects.
- `403` : commune, rôle ou origine refusée.
- `404` : publication inexistante ou privée non accessible.
- `409` : adresse e-mail déjà utilisée.
- `413` : requête trop volumineuse.
- `415` : contenu autre que JSON à travers le relais.
- `422` : structure JSON refusée par l'extracteur Rust, notamment en cas de champs supplémentaires.
- `503` : backend inaccessible depuis Next.js.

Les erreurs d'extraction de requête produites par Axum peuvent être des réponses texte sur l'API Rust directe. Le relais Next.js les convertit en JSON avec un message français et conserve leur code HTTP.
