# Déploiement du prototype sur OVHcloud

Ce déploiement conserve **le mode prototype** : inscriptions de maires non vérifiées et données initiales fictives. FranceConnect et la validation des mandats restent à intégrer avant la version définitive. `NODE_ENV=production` optimise Next.js ; il ne modifie pas le statut métier du prototype.

## Architecture

Une instance Ubuntu 24.04 héberge quatre conteneurs : Caddy (HTTPS), Next.js, l’API Rust et MongoDB 8. Seuls les ports 80 et 443 sont publiés. MongoDB possède un compte applicatif limité à sa base. Les données MongoDB et les certificats sont conservés dans des volumes Docker. Les journaux sont limités à trois fichiers de 10 Mo par service.

Les images sont construites avec les fichiers de verrouillage du dépôt. Les images de base suivent leur branche de version : tester les reconstructions et les mises à jour avant de les appliquer au serveur.

## Première installation

Copier le dépôt sur le serveur, puis depuis sa racine :

```sh
sudo bash deploy/bootstrap-ubuntu.sh
python3 deploy/configure.py
sudo docker compose -f compose.ovh.yaml config --quiet
sudo docker compose -f compose.ovh.yaml up --build -d
sudo docker compose -f compose.ovh.yaml ps
```

Le script de configuration crée `.env` avec des secrets aléatoires et des permissions `0600`. Il refuse de remplacer un fichier existant. Conserver ce fichier en lieu sûr, hors de Git ; ne pas copier les secrets de développement sur le serveur. L’administrateur utilise la valeur `QOMUNE_ADMIN_PASSWORD` de ce fichier, qui est propre au serveur. Les mots de passe des comptes fictifs doivent être communiqués séparément par le responsable du prototype ; ils ne sont pas affichés dans les formulaires et ces comptes ne doivent pas être utilisés pour des informations réelles.

Le domaine `QOMUNE_DOMAIN` doit pointer vers l’IPv4 du serveur. Ne configurer une entrée AAAA que lorsque la connectivité IPv6 a été vérifiée. Ouvrir 80/443 pour le site et réserver autant que possible SSH aux adresses d’administration. Caddy obtient et renouvelle le certificat automatiquement après propagation du DNS. Les messageries du domaine ne nécessitent aucune modification.

## Vérifications

```sh
sudo docker compose -f compose.ovh.yaml ps
curl --fail https://qomune.fr/api/health
curl --fail https://qomune.fr/api/communes/meta
```

Vérifier également la page publique, le téléchargement du cahier des charges, le favicon, la connexion d’un élu et la démonstration publique à `/administration` et la connexion privée à `/administration/connexion`. Sans session administrateur, `/api/admin/statistics` doit répondre `401` ; les publications et échanges privés ne doivent pas être exposés au visiteur.

## Mise à jour

Sauvegarder la base avant une mise à jour. Actualiser les sources dans le répertoire du serveur, puis relancer `sudo docker compose -f compose.ovh.yaml up --build -d`. La base existante est conservée. Le script MongoDB d’initialisation ne s’exécute que sur un volume neuf. Modifier le mot de passe MongoDB dans `.env` ne suffit pas à modifier le compte déjà présent en base.

## Sauvegardes

La sauvegarde OVHcloud de l’instance est activée quotidiennement, avec sept entrées conservées. Elle est facturée selon le volume sauvegardé. Contrôler son exécution dans « Instance Backup » et surveiller la consommation dans « Billing Control » : un budget annoncé n’est pas un plafond automatique de facturation.

Une sauvegarde applicative MongoDB peut être créée avec :

```sh
mkdir -p backups
chmod 700 backups
sudo docker compose -f compose.ovh.yaml exec -T mongodb sh -c \
  'mongodump --username "$MONGO_INITDB_ROOT_USERNAME" --password "$MONGO_INITDB_ROOT_PASSWORD" --authenticationDatabase admin --db qomune --archive --gzip' \
  > "backups/qomune-$(date -u +%Y%m%dT%H%M%SZ).archive.gz"
chmod 600 backups/*.archive.gz
```

Copier les sauvegardes vers un emplacement privé distinct du serveur et tester une restauration sur une instance séparée. Ne pas lancer de suppression de volume pendant une mise à jour.

## Sources de configuration

- [Installation officielle Docker sur Ubuntu](https://docs.docker.com/engine/install/ubuntu/)
- [Serveur autonome Next.js](https://nextjs.org/docs/app/api-reference/config/next-config-js/output)
- [HTTPS automatique Caddy](https://caddyserver.com/docs/automatic-https)

## Activer le suivi des pages vues sur un serveur déjà installé

```sh
python3 deploy/configure.py --add-metrics-key
sudo docker compose -f compose.ovh.yaml up --build -d api web
```

La migration ajoute seulement la clé de mesure manquante dans `.env` et conserve tous les secrets existants. Le graphique des visites est réservé à `/administration/connexion`. Il agrège les chargements HTML de tous les visiteurs, sans cookies de suivi, IP ou identité. Les métriques ne sont pas des visiteurs uniques ; les préchargements et les contrôles de santé ne comptent pas. Les journées précédant l'activation sont indiquées comme non mesurées. La collecte n'empêche pas l'ouverture du site en cas d'indisponibilité de la mesure.
