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

L'identité de l'auteur, son rôle et sa commune sont déterminés par la session. Les champs supplémentaires sont refusés. Réponse : `201` avec `{ "id": "..." }`.

Le relais Next.js exige une origine identique à celle de la requête ainsi qu'un contenu `application/json` pour les écritures. Il n'accepte que les routes énumérées ci-dessus. Taille maximale d'une requête : 64 Kio.

## Erreurs

Les erreurs métier prennent la forme `{ "error": "Message en français" }`.

- `400` : contenu ou filtre invalide.
- `401` : connexion requise ou identifiants incorrects.
- `403` : commune ou origine refusée.
- `404` : publication inexistante ou privée non accessible.
- `413` : requête trop volumineuse.
- `415` : contenu autre que JSON à travers le relais.
- `422` : structure JSON refusée par l'extracteur Rust, notamment en cas de champs supplémentaires.
- `503` : backend inaccessible depuis Next.js.

Les erreurs d'extraction de requête produites par Axum peuvent être des réponses texte sur l'API Rust directe. Le relais Next.js les convertit en JSON avec un message français et conserve leur code HTTP.
