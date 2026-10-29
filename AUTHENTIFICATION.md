# Inscription des maires : prototype et production

## Fonctionnement du prototype

L'inscription du maire est **déclarative et non vérifiée**. Le formulaire accessible par **Espace élus → Maire : créer mon compte** demande un nom complet, une commune du référentiel, une adresse e-mail et un mot de passe d'au moins 12 caractères.

Le serveur crée un compte de rôle `maire`, enregistre le statut interne `verification_status=unverified_prototype` et ouvre sa session. Il ne contrôle ni l'identité, ni le mandat, ni la propriété de l'adresse e-mail. Le mot de passe est haché avec Argon2.

Un index MongoDB réserve un seul compte de maire par commune. Une adresse e-mail déjà utilisée ou une commune ayant déjà un maire inscrit produit une réponse `409`, sans remplacer le compte existant. Les six communes de démonstration disposent déjà de leurs comptes préconfigurés.

Le maire peut publier et ajouter les membres de son conseil. Chaque membre ne peut consulter les publications privées que de la commune associée à sa session.

Le mode par défaut est `QOMUNE_MODE=prototype`. Toute autre valeur empêche le démarrage du serveur : la production n'est pas encore disponible. L'API d'inscription refuse également l'inscription déclarative hors du prototype. Aucune connexion FranceConnect n'est simulée.

## Parcours prévu pour la production

Deux contrôles complémentaires sont prévus :

1. **FranceConnect** : authentifier la personne et obtenir les données d'identité nécessaires, selon l'habilitation du service.
2. **Répertoire national des élus (RNE)** : rapprocher l'identité authentifiée du maire référencé pour le code INSEE de la commune choisie.

L'identité fournie par FranceConnect ne doit pas, à elle seule, attribuer le rôle de maire. Le rapprochement métier doit vérifier le mandat avec une extraction suffisamment récente du RNE. Les homonymes, divergences de nom, absences et données en cours d'actualisation nécessitent un état d'attente et un traitement explicite ; une simple ressemblance de nom ne suffit pas pour ouvrir les droits.

Cette intégration est **prévue, mais non implémentée**. Aucun compte issu de l'inscription libre du prototype ne doit être considéré comme vérifié en production. Il faudra prévoir sa validation et révoquer ses anciennes sessions avant de lui ouvrir les droits de production.

## Référentiel officiel des mandats

Le [Répertoire national des élus sur data.gouv.fr](https://www.data.gouv.fr/datasets/repertoire-national-des-elus-1) est publié par le ministère de l'Intérieur. La ressource **`elus-maires-mai.csv`**, identifiant `2876a346-d50c-4911-934e-19ee07b0e503`, fournit les maires. Elle est distincte des fichiers des conseillers municipaux et des extractions d'élus sortants.

Lors de la consultation du 29 octobre 2026, la ressource publiée est datée du **11 août 2026**. Son en-tête fournit notamment : code et libellé de la commune, nom et prénom de l'élu, date de naissance, date de début du mandat et date de début de la fonction. Les données du RNE sont publiées trimestriellement ; la fiche signale encore certaines actualisations en cours après les élections de mars 2026.

La provenance vérifiée est conservée dans `backend/reference/mayors-source.json`. Ce fichier décrit une source **prévue pour la production** : les données nominatives des maires n'ont pas été importées et aucun rapprochement RNE n'est exécuté dans le prototype. Le futur import devra résoudre la ressource courante via l'API data.gouv.fr, contrôler son schéma et conserver sa date et son empreinte. Ne pas utiliser durablement l'URL horodatée comme une source automatiquement à jour.

Le RNE comporte également **`elus-conseillers-municipaux-cm.csv`**, ressource `d5f400de-ae3f-4966-8cb6-a85c70c6c24a`. L'en-tête consulté fournit le code et le libellé de la commune, le nom et le prénom, la date de début du mandat, le libellé de la fonction et sa date de début. Ce fichier permet donc aussi de contrôler l'appartenance d'un conseiller au conseil municipal d'une commune. Un élu peut apparaître plusieurs fois lorsqu'il exerce plusieurs fonctions ; le futur rapprochement devra gérer ces lignes sans créer plusieurs comptes. Dans le prototype, les conseillers restent ajoutés par le maire, sans contrôle RNE.

Ne conserver que les informations nécessaires au contrôle du mandat et à sa traçabilité ; les autres colonnes de l'extraction ne sont pas nécessaires à ce parcours.

## Documentation officielle

- [Fonctionnement de FranceConnect](https://www.franceconnect.gouv.fr/)
- [Étapes pour devenir fournisseur de service](https://docs.partenaires.franceconnect.gouv.fr/fs/devenir-fs/pilotage-etapes/)
- [Données d'identité et habilitation FranceConnect](https://docs.partenaires.franceconnect.gouv.fr/fs/devenir-fs/projet-datapass/)
- [Répertoire national des élus — ministère de l'Intérieur](https://www.data.gouv.fr/datasets/repertoire-national-des-elus-1)
