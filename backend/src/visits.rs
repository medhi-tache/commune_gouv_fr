//! Compteurs de pages quotidiens, sans IP, cookie, identité ni historique individuel.
use super::*;
use sha2::{Digest, Sha256};

pub async fn initialize(database: &Database) -> mongodb::error::Result<()> {
    database
        .collection::<Document>("visit_tracking")
        .update_one(
            doc! {"_id":"settings"},
            doc! {"$setOnInsert":{"started_at":Utc::now().to_rfc3339()}},
        )
        .upsert(true)
        .await?;
    database
        .collection::<Document>("daily_visits")
        .create_index(
            IndexModel::builder()
                .keys(doc! {"expires_at":1})
                .options(
                    IndexOptions::builder()
                        .expire_after(std::time::Duration::ZERO)
                        .build(),
                )
                .build(),
        )
        .await?;
    Ok(())
}
fn fingerprint(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
/// Ce secret est partagé seulement par Next.js côté serveur et l'API interne.
pub async fn configure(database: &Database, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !(32..=200).contains(&key.len()) {
        return Err("La clé de mesure doit contenir entre 32 et 200 caractères.".into());
    }
    database
        .collection::<Document>("visit_tracking")
        .update_one(
            doc! {"_id":"settings"},
            doc! {"$set":{"ingest_key_hash":fingerprint(key)}},
        )
        .await?;
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Visit {
    page: String,
}
/// Cette route n'est pas autorisée dans le relais public /api ; la clé est vérifiée en complément.
pub async fn record(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<Visit>,
) -> ApiResult<StatusCode> {
    let supplied = headers
        .get("x-qomune-metrics-key")
        .and_then(|h| h.to_str().ok())
        .filter(|value| (32..=200).contains(&value.len()))
        .ok_or_else(|| ApiError(StatusCode::FORBIDDEN, "Mesure non autorisée.".into()))?;
    let config = database
        .collection::<Document>("visit_tracking")
        .find_one(doc! {"_id":"settings"})
        .await?;
    if config
        .as_ref()
        .and_then(|d| d.get_str("ingest_key_hash").ok())
        != Some(fingerprint(supplied).as_str())
    {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Mesure non autorisée.".into(),
        ));
    }
    if !["public", "admin_demo", "admin_private"].contains(&input.page.as_str()) {
        return Err(invalid("Page inconnue."));
    }
    let today = Utc::now().date_naive();
    let date = today.to_string();
    let expires = DateTime::from_millis(
        (today + Duration::days(91))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis(),
    );
    let collection = database.collection::<Document>("daily_visits");
    let mut increment = doc! {};
    increment.insert(&input.page, 1_i64);
    let update = doc! {"$inc":increment,"$setOnInsert":{"date":&date,"expires_at":expires}};
    if let Err(error) = collection
        .update_one(doc! {"_id":&date}, update.clone())
        .upsert(true)
        .await
    {
        // Deux premières visites peuvent créer le même jour simultanément ; on rejoue sans upsert.
        if matches!(error.kind.as_ref(),mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(write)) if write.code==11000)
        {
            collection
                .update_one(
                    doc! {"_id":&date},
                    doc! {"$inc":update.get_document("$inc").unwrap().clone()},
                )
                .await?;
        } else {
            return Err(error.into());
        }
    }
    Ok(StatusCode::NO_CONTENT)
}
fn count(row: &Document, field: &str) -> i64 {
    row.get_i64(field)
        .or_else(|_| row.get_i32(field).map(i64::from))
        .unwrap_or(0)
}
/// Les jours antérieurs au début de mesure sont distingués des jours mesurés sans visite.
pub async fn summary(
    database: &Database,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> ApiResult<Value> {
    let config = database
        .collection::<Document>("visit_tracking")
        .find_one(doc! {"_id":"settings"})
        .await?;
    let started_at = config
        .as_ref()
        .and_then(|d| d.get_str("started_at").ok())
        .unwrap_or("")
        .to_owned();
    let first_day = started_at.get(..10).unwrap_or("9999-12-31");
    let rows: Vec<Document> = database
        .collection("daily_visits")
        .find(doc! {"_id":{"$gte":start.to_string(),"$lte":end.to_string()}})
        .await?
        .try_collect()
        .await?;
    let mut timeline = Vec::new();
    let mut total = 0;
    for index in 0..=(end - start).num_days() {
        let date = (start + Duration::days(index)).to_string();
        let row = rows
            .iter()
            .find(|r| r.get_str("_id").ok() == Some(date.as_str()));
        let public = row.map(|r| count(r, "public")).unwrap_or(0);
        let demo = row.map(|r| count(r, "admin_demo")).unwrap_or(0);
        let private = row.map(|r| count(r, "admin_private")).unwrap_or(0);
        total += public + demo + private;
        timeline.push(json!({"date":date,"public":public,"admin_demo":demo,"admin_private":private,"measured":date.as_str()>=first_day}));
    }
    Ok(json!({"started_at":started_at,"total":total,"timeline":timeline}))
}
