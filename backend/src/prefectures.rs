//! Référentiel et comptes institutionnels fictifs des 101 préfectures départementales.
use super::*;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Directory {
    checked_at: String,
    members: Vec<Prefecture>,
}
#[derive(Deserialize)]
struct Prefecture {
    slug: String,
    name: String,
    role: String,
    ministries: Vec<String>,
    department_code: String,
    department_name: String,
}

/// Le code administratif, et non le nom du département, borne les diffusions.
pub fn department_code(commune_id: &str) -> Option<&str> {
    if commune_id.len() != 5 || !commune_id.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }
    commune_id.get(..if commune_id.starts_with("97") { 3 } else { 2 })
}

/// Un secret d'installation privé produit un mot de passe distinct par préfecture.
/// Les mots de passe ne sont ni inclus dans le référentiel public ni renvoyés par une API.
pub fn demo_password(secret: &str, code: &str) -> String {
    let bytes = Sha256::digest(format!("qomune-prefecture-demo:{code}:{secret}").as_bytes());
    bytes[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Une réinitialisation conserve les mots de passe et les publications existants.
pub async fn initialize(
    database: &Database,
    secret: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if secret.len() < 32 {
        return Err(
            "Le secret privé des comptes de préfecture doit contenir au moins 32 caractères."
                .into(),
        );
    }
    let directory: Directory = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../frontend/reference/prefectures.json"
    )))?;
    let accounts = database.collection::<Document>("government_demo_accounts");
    for person in directory.members {
        let id = format!("demo-prefecture-{}", person.slug);
        let existing = accounts.find_one(doc! {"id":&id}).await?;
        let mut insert = doc! {"id":&id};
        if existing.is_none() {
            let password = demo_password(secret, &person.department_code);
            let hash = tokio::task::spawn_blocking(move || {
                let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes()).expect("Sel valide");
                Argon2::default()
                    .hash_password(password.as_bytes(), &salt)
                    .map(|hash| hash.to_string())
                    .map_err(|_| "Création du secret fictif impossible.")
            })
            .await??;
            insert.insert("password_hash", hash);
        }
        accounts.update_one(doc! {"id":&id}, doc! {
            "$set": {
                "slug":&person.slug,"name":person.name,"function":person.role,
                "ministries":person.ministries,"email":format!("{}@prefecture.demo.qomune.fr",person.slug),
                "institution":"prefecture","is_demo":true,"source_checked_at":&directory.checked_at,
                "department_code":person.department_code,"department_name":person.department_name
            },
            "$setOnInsert":insert
        }).upsert(true).await?;
    }
    Ok(())
}
