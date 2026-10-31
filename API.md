# API du prototype

Préfixe navigateur : `/api`. Le serveur Rust expose les mêmes chemins sans ce préfixe sur `127.0.0.1:8080`. Les réponses contiennent du JSON. Les données du relais Next.js ne sont pas mises en cache.

## Routes

| Méthode | Chemin | Comportement |
| --- | --- | --- |
| GET | `/health` | État du serveur. |
| GET | `/communes` | Recherche bornée dans les 34 875 communes du COG 2026. |
| GET | `/communes/meta` | Nombre de communes, millésime, source et date du dernier import terminé. |
| POST | `/auth/register` | Inscription non vérifiée d’un maire, réservée au prototype ; ouvre une session. |
| POST | `/auth/login` | Connexion par `email` et `password` ; pose un cookie de session. |
| POST | `/auth/logout` | Révoque la session et efface son cookie. Corps : `{}`. |
| GET | `/auth/me` | `{ "member": ... }` ou `{ "member": null }`. |
| GET | `/members` | Liste des membres de sa commune, réservée au maire. |
| POST | `/members` | Création d’un conseiller municipal par le maire de la commune. |
| GET | `/publications` | Fil public par défaut ; au maximum 100 résultats, du plus récent au plus ancien. |
| GET | `/publications/{id}` | Publication publique, ou publication privée de sa propre commune. |
| POST | `/publications` | Création par un maire ou conseiller connecté. |
| GET | `/publications/{id}/discussion` | Commentaires et réactions, uniquement pour les membres du conseil de cette commune. |
| POST | `/publications/{id}/comments` | Ajouter un commentaire interne au conseil. |
| POST | `/publications/{id}/reaction` | Définir ou retirer sa propre réaction interne. |

## Rechercher une commune

`GET /communes` accepte :

- `q` : nom ou début de code INSEE, au plus 200 octets UTF-8 ; recherche des noms insensible à la casse, aux accents et à la ponctuation séparatrice.
- `limit` : de 1 à 100, valeur par défaut 20. La réponse reste une liste JSON de communes.
- `ids` : liste de codes INSEE séparés par des virgules, au plus 200 codes. Ce mode renvoie tous les codes trouvés de la liste et sert à afficher les noms des communes présentes dans le fil.

Chaque commune contient `id`, `name`, `department` et `region`. Les codes sont des chaînes, jamais des nombres. Les champs internes de normalisation et de provenance ne sont pas exposés dans cette liste. La recherche ne réalise aucun téléchargement vers data.gouv.fr : elle lit le référentiel importé dans MongoDB.

`GET /communes/meta` renvoie `{ "total": 34875, "vintage": 2026, "source_url": "...", "imported_at": "..." }` après import. Avant un import terminé, le millésime, la source et la date peuvent être `null`.

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

## Inscrire un maire dans le prototype

`POST /auth/register` reçoit uniquement `name`, `email`, `password` et `commune_id`. Le nom contient 2 à 120 caractères ; l'adresse e-mail usuelle ASCII est normalisée en minuscules et limitée à 200 caractères. Le mot de passe contient au moins 12 caractères et au plus 200 octets UTF-8. La commune doit exister dans le référentiel.

Le serveur impose le rôle `maire` et enregistre `verification_status=unverified_prototype`. Il ne vérifie ni l'identité ni le mandat. Un seul maire est autorisé par commune, même en cas de demandes simultanées ; les doublons d'adresse ou de commune renvoient `409`. Le rôle et les champs de vérification ne peuvent pas être fournis par le navigateur.

La réponse `201` contient `{ "ok": true }` et pose le cookie de session ; `/auth/me` donne le profil. L'inscription est refusée hors du mode prototype. Le parcours FranceConnect + RNE prévu en production n'est pas encore disponible ; voir `AUTHENTIFICATION.md`.

## Échanges internes au conseil

Les trois routes d'échange exigent une session de maire ou de conseiller de la commune de la publication. Ce contrôle s'applique même si la publication principale est publique : un visiteur reçoit `401`, un membre d'une autre commune `404`. Les listes et lectures publiques de publications ne renvoient ni échanges, ni noms de réacteurs, ni compteurs internes.

`GET /publications/{id}/discussion` renvoie :

- `comments` : les 50 derniers commentaires, présentés dans l'ordre chronologique ;
- `comments_count` : le nombre total de commentaires sur cette publication ;
- `next_cursor` : un curseur opaque pour charger les commentaires précédents avec `?before=...`, ou `null` ;
- `reactions` : `counts` par type, `total`, `mine` pour la réaction de la session ou `null`, et `members` avec les noms, fonctions et réactions des participants.

`POST /publications/{id}/comments` accepte uniquement `{ "body": "Mon commentaire" }`. Le texte doit contenir 1 à 2 000 caractères après suppression des espaces aux extrémités. Le serveur renseigne l'auteur, sa commune et la date. Réponse `201` : le commentaire créé. Le contenu est affiché comme texte, sans interprétation HTML.

`POST /publications/{id}/reaction` accepte uniquement `{ "reaction": "like" }`. Valeurs : `like`, `love`, `haha`, `wow`, `sad`, `angry`, ou `null` pour retirer sa réaction. L'envoi répété du même type ne crée pas de doublon ; l'interface envoie explicitement `null` lorsqu'un membre clique de nouveau sur sa réaction. La réponse `200` contient le nouvel objet de synthèse `reactions`. Un index unique garantit une seule réaction par membre et publication, même en cas d'accès simultanés.

Aucun identifiant d'auteur, rôle ou commune n'est accepté dans les corps d'écriture. Il n'y a pas de notification ou de diffusion en temps réel ; la modification, la suppression et la modération des commentaires ne sont pas encore implémentées.

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
- `409` : adresse e-mail déjà utilisée ou maire déjà inscrit pour la commune.
- `413` : requête trop volumineuse.
- `415` : contenu autre que JSON à travers le relais.
- `422` : structure JSON refusée par l'extracteur Rust, notamment en cas de champs supplémentaires.
- `503` : backend inaccessible depuis Next.js.

Les erreurs d'extraction de requête produites par Axum peuvent être des réponses texte sur l'API Rust directe. Le relais Next.js les convertit en JSON avec un message français et conserve leur code HTTP.

### Publications publiques d’un membre du Gouvernement

`GET /government/{slug}/publications` est accessible sans session. Il renvoie
`{ "publications": [...], "next_cursor": "..." }` avec 20 messages publics de
démonstration au maximum, du plus récent au plus ancien. Les diffusions nationales
et celles destinées à une commune précise apparaissent sur le profil de leur auteur.

Pour lire la suite, transmettre `?before=<next_cursor>`. Un curseur doit désigner
une publication publique du même compte ; les curseurs d’un autre auteur ou d’un
message privé sont refusés. `next_cursor` vaut `null` en fin de liste. Aucun champ
technique d’authentification n’est renvoyé. Les autres méthodes HTTP ne sont pas
exposées par le relais web pour cette route publique.

### Préfectures de démonstration

Les comptes fictifs des préfectures utilisent les routes de session
`/government-demo/login`, `/government-demo/me`, `/government-demo/logout` et
`/government-demo/publications`, avec `institution: "prefecture"`. Ces noms de
routes sont conservés pour rester compatibles avec les autres comptes fictifs.

`GET /prefectures/{slug}/publications` expose uniquement les publications
publiques de l’auteur, par pages de 20 avec le même curseur `before` que le
Gouvernement. `GET /communes?department=58&q=Nevers` borne la recherche au
département. Le filtre est appliqué avant la recherche et la limite.

Sans `commune_id`, un compte de préfecture produit `scope: "department"`, avec
`department_code` et `department_name` imposés par son compte côté serveur.
Avec `commune_id`, une commune d’un autre département est refusée (403).
Les messages départementaux apparaissent dans chacun des fils communaux
concernés et dans le fil public global. Aucune route de diffusion privée ni
aucun mot de passe ne sont exposés par ces API publiques.

### Carburants les moins chers de la commune

`GET /carburants/{code_insee}` est public et renvoie `commune_id`, `commune_name`,
`station_count` (stations locales), `nearby_radius_km`, `fetched_at`, `source_url` et `prices`. Pour chacun des six carburants,
`price_per_litre` contient le minimum déclaré en euros par litre (ou `null`) et
`stations` liste les ex æquo avec leur adresse, ville, `updated_at` et
`older_than_week`. Les minima sont comparés au millième d’euro.

Le code doit désigner une commune du référentiel de Qomune. Réponses : `400` pour
un code mal formé, `404` pour une commune inconnue, `503` si les données officielles
ne peuvent pas être récupérées. Seule la lecture est autorisée. Le relais Next.js
accorde jusqu’à 45 secondes à cette route lors d’une récupération initiale.
Aucun cookie ni identifiant du visiteur n’est envoyé aux services de données.

En l’absence de prix local pour un carburant, le comparateur cherche le minimum dans les stations voisines jusqu’à 25 km à vol d’oiseau du centre approximatif de l’emprise communale. La priorité reste au prix local, même si un voisin est moins cher. Chaque résultat indique `search_scope` (`commune`, `nearby`, `none`) ; les stations voisines comportent une `distance_km` indicative. Sans résultat dans ce rayon, aucun prix n’est inventé.
