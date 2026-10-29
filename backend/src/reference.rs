//! Référentiel COG 2026 de l'Insee, téléchargé depuis les ressources publiées sur data.gouv.fr.
//! Les fichiers sont embarqués pour que les démarrages restent possibles sans accès réseau.
use super::*;
use futures_util::{StreamExt, stream};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

pub const SOURCE_URL: &str = "https://www.data.gouv.fr/datasets/code-officiel-geographique-1";
const SOURCE_TAG: &str = "insee-cog-2026";
const DATA: &str = include_str!("../reference/communes-2026.json");
const MANIFEST: &str = include_str!("../reference/source.json");

#[derive(Deserialize, Clone)]
pub struct ReferenceCommune {
    #[serde(flatten)]
    pub commune: Commune,
    pub search_name: String,
}
#[derive(Deserialize)]
pub struct Manifest {
    pub vintage: i32,
    pub count: usize,
    pub sha256: String,
}

/// Recherche française insensible aux accents, à la casse, aux tirets et aux apostrophes.
pub fn normalize_name(value: &str) -> String {
    value
        .to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Valide entièrement les fichiers avant toute écriture : millésime, empreinte, unicité et codes.
pub fn load() -> Result<(Vec<ReferenceCommune>, Manifest), String> {
    let manifest: Manifest =
        serde_json::from_str(MANIFEST).map_err(|_| "Métadonnées COG invalides")?;
    if manifest.vintage != 2026
        || Sha256::digest(DATA.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            != manifest.sha256
    {
        return Err("L'empreinte du référentiel COG est invalide".into());
    }
    let records: Vec<ReferenceCommune> =
        serde_json::from_str(DATA).map_err(|_| "Référentiel COG invalide")?;
    let mut ids = HashSet::new();
    for record in &records {
        let commune = &record.commune;
        let code = commune.id.as_bytes();
        if code.len() != 5
            || !(code.iter().all(u8::is_ascii_digit)
                || ((commune.id.starts_with("2A") || commune.id.starts_with("2B"))
                    && code[2..].iter().all(u8::is_ascii_digit)))
            || commune.name.trim().is_empty()
            || commune.department.trim().is_empty()
            || commune.region.trim().is_empty()
            || !ids.insert(commune.id.clone())
            || record.search_name != normalize_name(&commune.name)
        {
            return Err("Commune COG invalide, dupliquée ou mal normalisée".into());
        }
    }
    if records.len() != manifest.count || records.len() < 34000 {
        return Err("Nombre de communes COG incohérent".into());
    }
    Ok((records, manifest))
}

/// Met à jour les seuls documents communaux. Comptes, mandats et publications restent liés à leur code INSEE.
pub async fn import_records(
    database: &Database,
    records: Vec<ReferenceCommune>,
    manifest: &Manifest,
) -> mongodb::error::Result<usize> {
    let previous = database
        .collection::<Document>("reference_imports")
        .find_one(doc! {"_id":"communes"})
        .await?;
    let current_count = database
        .collection::<Document>("communes")
        .count_documents(doc! {"source":SOURCE_TAG})
        .await?;
    if previous.as_ref().and_then(|d| d.get_str("sha256").ok()) == Some(manifest.sha256.as_str())
        && current_count == records.len() as u64
    {
        return Ok(records.len());
    }
    database
        .collection::<Document>("communes")
        .create_index(
            IndexModel::builder()
                .keys(doc! {"search_name":1,"id":1})
                .build(),
        )
        .await?;
    let count = records.len();
    // Le parallélisme est borné pour éviter de saturer MongoDB pendant le premier import.
    stream::iter(records)
        .map(|record| async move {
            let c = record.commune;
            database.collection::<Document>("communes").update_one(doc!{"id":&c.id},doc!{"$set":{
            "id":&c.id,"name":c.name,"department":c.department,"region":c.region,
            "search_name":record.search_name,"source":SOURCE_TAG,"vintage":manifest.vintage
        }}).upsert(true).await
        })
        .buffer_unordered(32)
        .try_collect::<Vec<_>>()
        .await?;
    // Ce marqueur n'est écrit qu'une fois tous les documents enregistrés ; un import interrompu se reprend.
    database.collection::<Document>("reference_imports").update_one(doc!{"_id":"communes"},doc!{"$set":{
        "vintage":manifest.vintage,"count":count as i64,"sha256":&manifest.sha256,"source_url":SOURCE_URL,"imported_at":Utc::now().to_rfc3339()
    }}).upsert(true).await?;
    Ok(count)
}

pub async fn import(database: &Database) -> Result<usize, Box<dyn std::error::Error>> {
    let (records, manifest) = load()?;
    Ok(import_records(database, records, &manifest).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn referentiel_officiel_complet_codes_et_accents_conserves() {
        let (records, manifest) = load().unwrap();
        assert_eq!(records.len(), 34875);
        assert_eq!(manifest.vintage, 2026);
        for code in [
            "01001", "2A004", "2B033", "75056", "97101", "97201", "97301", "97411", "97611",
        ] {
            assert!(
                records.iter().any(|r| r.commune.id == code),
                "Code absent : {code}"
            );
        }
        assert!(
            !records.iter().any(|r| r.commune.id == "75101"),
            "Un arrondissement n'est pas une commune autonome"
        );
        assert_eq!(normalize_name("L’Haÿ-les-Roses"), "l hay les roses");
        assert_eq!(normalize_name("SAINT-ÉTIENNE"), "saint etienne");
        assert_eq!(
            records
                .iter()
                .find(|r| r.commune.id == "74010")
                .unwrap()
                .commune
                .name,
            "Annecy"
        );
    }
}
