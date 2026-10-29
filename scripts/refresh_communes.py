#!/usr/bin/env python3
"""Télécharge le COG 2026 publié par l'Insee sur data.gouv.fr, sans dépendance Python externe."""
import csv
import hashlib
import io
import json
from pathlib import Path
import unicodedata
from urllib.parse import urlparse
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent.parent
CATALOG = "https://www.data.gouv.fr/api/1/datasets/code-officiel-geographique-1/"
PAGE = "https://www.data.gouv.fr/datasets/code-officiel-geographique-1"
RESOURCES = {
    "communes": "c5800591-813b-4ce2-9a3f-739a9c5d9558",
    "departements": "e5f32177-5841-4a46-9c17-f8ff94bd38fc",
    "regions": "f7057d4e-0078-45de-95b7-2a0ce6594807",
}


def download(url):
    """Limite les téléchargements aux sources HTTPS attendues et à 10 Mio par fichier."""
    def allowed(value):
        parsed = urlparse(value)
        return parsed.scheme == "https" and parsed.hostname in {"www.data.gouv.fr", "www.insee.fr"}
    if not allowed(url):
        raise ValueError("Source de téléchargement non autorisée")
    with urlopen(Request(url, headers={"User-Agent": "Qomune-prototype/0.1"}), timeout=60) as response:
        if not allowed(response.url):
            raise ValueError("Destination de redirection non autorisée")
        content = response.read(10 * 1024 * 1024 + 1)
    if len(content) > 10 * 1024 * 1024:
        raise ValueError("Fichier source trop volumineux")
    return content


def normalized_name(value):
    value = "".join(c for c in unicodedata.normalize("NFD", value.lower()) if not unicodedata.combining(c))
    return " ".join("".join(c if c.isalnum() else " " for c in value).split())


def main():
    catalog = json.loads(download(CATALOG))
    if catalog["id"] != "58c984b088ee386cdb1261f3":
        raise ValueError("Jeu de données inattendu")
    tables, sources = {}, {}
    for name, resource_id in RESOURCES.items():
        resource = next(r for r in catalog["resources"] if r["id"] == resource_id)
        if "2026" not in resource["title"] or resource["format"] != "csv":
            raise ValueError("Millésime ou format inattendu")
        raw = download(resource["url"])
        tables[name] = list(csv.DictReader(io.StringIO(raw.decode("utf-8-sig"))))
        sources[name] = {"resource_id": resource_id, "title": resource["title"], "url": resource["url"], "sha256": hashlib.sha256(raw).hexdigest()}
    departments = {r["DEP"]: r["LIBELLE"] for r in tables["departements"]}
    regions = {r["REG"]: r["LIBELLE"] for r in tables["regions"]}
    # COM désigne une commune actuelle. COMD, COMA et ARM ne sont pas des communes autonomes.
    communes = [{"id": r["COM"], "name": r["LIBELLE"],
                 "department": f'{departments[r["DEP"]]} · {r["DEP"]}', "region": regions[r["REG"]],
                 "search_name": normalized_name(r["LIBELLE"])}
                for r in tables["communes"] if r["TYPECOM"] == "COM"]
    communes.sort(key=lambda r: r["id"])
    if len(communes) < 34000 or len({r["id"] for r in communes}) != len(communes):
        raise ValueError("Référentiel incomplet ou codes dupliqués : aucun fichier n'a été remplacé")
    content = json.dumps(communes, ensure_ascii=False, separators=(",", ":")) + "\n"
    manifest = {"vintage": 2026, "count": len(communes), "producer": "Insee", "license": "Licence Ouverte / Open Licence",
                "source_url": PAGE, "dataset_id": catalog["id"], "sha256": hashlib.sha256(content.encode()).hexdigest(),
                "scope": "France métropolitaine et DROM ; TYPECOM=COM", "resources": sources}
    directory = ROOT / "backend" / "reference"
    directory.mkdir(parents=True, exist_ok=True)
    # Les deux fichiers sont préparés après validation complète ; aucun accès MongoDB n'est effectué ici.
    (directory / "communes-2026.json").write_text(content, encoding="utf-8")
    (directory / "source.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f'{len(communes):,} communes actuelles téléchargées depuis le COG 2026 référencé sur data.gouv.fr.'.replace(",", " "))


if __name__ == "__main__":
    main()
