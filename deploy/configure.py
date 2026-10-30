#!/usr/bin/env python3
"""Crée une configuration privée sans afficher les secrets et sans remplacer un fichier existant."""
from pathlib import Path
import os
import secrets

root = Path(__file__).resolve().parents[1]
path = root / ".env"
values = {
    "QOMUNE_DOMAIN": "qomune.fr",
    "MONGO_ROOT_PASSWORD": secrets.token_hex(32),
    "QOMUNE_DB_PASSWORD": secrets.token_hex(32),
    "QOMUNE_ADMIN_PASSWORD": secrets.token_urlsafe(32),
}
# La création exclusive empêche toute rotation involontaire des mots de passe MongoDB.
try:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
except FileExistsError:
    raise SystemExit("Configuration existante conservée : aucune modification.")
with os.fdopen(fd, "w") as file:
    file.write("# Secrets du serveur, à conserver hors du dépôt Git.\n")
    file.writelines(f"{key}={value}\n" for key, value in values.items())
print("Configuration privée créée dans .env ; aucun secret n’a été affiché.")
