# Guide du code

La documentation, les commentaires et les messages destinés aux utilisateurs sont rédigés en français. Les identifiants techniques suivent les conventions de Rust et de TypeScript.

## Organisation

- `backend/src/main.rs` : modèles, initialisation MongoDB, authentification, et routes.
- `backend/src/tests.rs` : tests d’intégration MongoDB et contrôle des accès.
- `backend/src/fixtures.rs` : élus et publications fictifs dans six communes de démonstration.
- `backend/src/discussion.rs` : commentaires internes et réactions, contrôle de commune et pagination.
- `frontend/components/council-discussion.tsx` : réactions, liste des participants et fil de commentaires.
- `backend/src/registration.rs` : inscription déclarative des maires, limitée au prototype.
- `frontend/components/mayor-registration.tsx` : formulaire d’inscription avec choix de commune.
- `AUTHENTIFICATION.md` : séparation entre le prototype et la vérification FranceConnect + RNE prévue en production.
- `backend/src/reference.rs` : validation et import idempotent du COG, normalisation des noms.
- `backend/reference/` : communes officielles normalisées et provenance des ressources.
- `scripts/refresh_communes.py` : téléchargement des ressources Insee publiées sur data.gouv.fr, filtrage de `TYPECOM=COM`.
- `frontend/components/commune-picker.tsx` : recherche distante de commune, limitée à 20 résultats.
- `frontend/app/api/[...path]/route.ts` : relais serveur Next.js vers Rust ; contrôle de l'origine des écritures et transmission du cookie de session.
- `frontend/components/council-members.tsx` : liste du conseil et ajout de comptes par le maire.
- `frontend/lib/api.ts` : requêtes du navigateur vers le relais Next.js.
- `frontend/components/dashboard.tsx` : fil de publications, filtres, lecture détaillée, connexion et création de publication.
- `frontend/app/globals.css` : styles partagés, cartes, formulaires et adaptation aux petits écrans.
- `scripts/dev.mjs` : lancement et arrêt des deux serveurs.

## Circulation des données

Le navigateur utilise uniquement `/api/...` sur le serveur Next.js. Ce dernier relaie les requêtes à l'API Rust, dont l'adresse reste interne. Le navigateur ne reçoit jamais le jeton de session dans un objet JavaScript : le cookie est `HttpOnly`, `SameSite=Strict`, avec une durée maximale de huit heures.

MongoDB conserve les données dans `communes`, `members`, `sessions` et `publications`. La collection `reference_imports` mémorise l’empreinte, la source et la date des imports terminés. Les index uniques portent sur les identifiants et les e-mails ; un index composé facilite la lecture par commune et visibilité. Un index TTL nettoie les sessions expirées, dont la validité est également vérifiée au moment de chaque accès. Chaque membre est rattaché à une seule commune dans ce prototype. La commune et le rôle sont lus dans la base à partir de la session, jamais depuis le formulaire de publication. Une reconnexion révoque la session précédente du navigateur ; la déconnexion supprime la session en base.

## Règles à préserver

1. Le fil public ne doit renvoyer que des publications publiques, même pour un utilisateur connecté.
2. Le fil du conseil exige une session et limite les résultats à la commune de son titulaire.
3. L'accès direct à une publication privée applique le même contrôle. Un accès refusé renvoie `404` pour ne pas confirmer l'existence du document.
4. Un auteur ne choisit pas son identité ni sa commune dans une requête. Les champs inconnus sont refusés à la création.
5. Les filtres BSON sont construits à partir de champs explicitement autorisés ; aucune requête MongoDB fournie par le navigateur n’est exécutée. Les caractères spéciaux de recherche sont échappés pour une recherche littérale. Le contenu utilisateur est rendu comme du texte par React, sans HTML injecté.
6. Seul un maire authentifié peut consulter la liste des membres et ajouter un conseiller à sa propre commune. Le rôle et la commune ne sont jamais acceptés dans le formulaire. Le mot de passe généré n’est renvoyé qu’à la création et seul son hash est stocké. Les doublons sont refusés par l’index unique sans modifier un compte existant.
7. Toute future pièce jointe doit être soumise aux mêmes contrôles côté serveur. Ne pas placer de document privé dans un répertoire statique public.

## Vérification

Démarrer MongoDB, puis depuis la racine :

```sh
npm run check
```

Cette commande vérifie le format Rust, lance les tests d’intégration de l’API contre MongoDB et construit le frontend avec vérification TypeScript.

Les tests utilisent des bases temporaires nommées `qomune_test_<UUID>`, indépendantes de `MONGODB_DATABASE`, puis les suppriment. Aucun test ne doit supprimer une base dont le nom vient directement de la configuration de l’application.

Les tests couvrent l'accès anonyme, les liens privés, l'isolation entre communes, la publication par une conseillère, le refus d'usurpation de commune, la validation, les filtres, la déconnexion, l'expiration et la conservation des données lors de l'initialisation. Ils couvrent aussi l’ajout réservé au maire, la connexion des nouveaux membres, le refus d’usurpation de rôle ou de commune, la validation des adresses et les doublons, y compris lors de créations simultanées.

## Étendre le prototype

Ajouter une route dans Rust puis, si nécessaire, l'autoriser explicitement dans le relais Next.js. Documenter son contrat dans `API.md`. Toute évolution des droits doit être accompagnée d'un test qui tente un accès depuis une autre commune.

Les collections et les index sont initialisés au démarrage. Les exemples utilisent des ajouts conditionnels pour résister à une initialisation interrompue. Avant une évolution réelle des documents, introduire des migrations versionnées. La recherche MongoDB est une recherche par sous-chaîne littérale, insensible à la casse, avec les caractères de regex échappés.

## Import du référentiel officiel

La préparation Python utilise uniquement la bibliothèque standard. Elle vérifie l’identifiant du jeu data.gouv.fr, le millésime et les domaines de téléchargement, puis combine communes, départements et régions. Rust contrôle l’empreinte SHA-256, le nombre de communes, les codes, leur unicité et la normalisation avant l’import. Les écritures MongoDB sont limitées à 32 opérations simultanées. Le marqueur de succès n’est écrit qu’à la fin ; un import interrompu est rejoué.

Les tests d’intégration utilisent un petit jeu isolé pour vérifier les filtres et la reprise sans charger 34 875 documents dans chacune de leurs bases temporaires. Un test dédié contrôle le fichier officiel complet, les codes corses, les zéros initiaux, les DROM et l’exclusion des arrondissements municipaux.

## Inscription déclarative

Les tests couvrent la création et la session d’un maire non vérifié, le choix d’une commune existante, les mots de passe trop courts, les champs supplémentaires, les doublons et les inscriptions simultanées. Le statut `unverified_prototype` ne doit jamais être traité comme une validation d’identité ou de mandat. Le serveur actuel refuse de démarrer hors de `QOMUNE_MODE=prototype` ; l’intégration FranceConnect et le rapprochement RNE restent à développer.

## Confidentialité des échanges du conseil

Les échanges ne sont pas des propriétés publiques d’une publication. Ils sont conservés dans `comments` et `reactions`, et ne sont lus que par les routes dédiées après contrôle de la session et de la commune. Cette règle s’applique aussi à une publication principale publique. Ne jamais enrichir une réponse du fil public avec ces compteurs ou ces contenus.

Les réactions ont un identifiant déterministe par publication et membre, ainsi qu’un index unique sur ce couple. Les mises à jour sont idempotentes ; la suppression vise uniquement la réaction de la session. Les commentaires sont paginés avec un couple date/identifiant pour éviter les pertes lorsque plusieurs messages ont la même date. Les tests couvrent l’accès anonyme et intercommunal, les signatures imposées, les réactions répétées et concurrentes, leur retrait, la pagination et la conservation lors de la recréation de l’application.
