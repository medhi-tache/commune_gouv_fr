//! Administration globale : authentification distincte et statistiques agrégées sans contenu privé.
use super::*;
use std::collections::BTreeMap;

/// Sans mot de passe configuré, aucun compte administrateur n'est créé.
/// Une modification du secret révoque les sessions existantes.
pub async fn configure(
    database: &Database,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if password.len() < 16 || password.len() > 200 {
        return Err(
            "Le mot de passe administrateur doit contenir entre 16 et 200 caractères.".into(),
        );
    }
    let config = database.collection::<Document>("administration");
    if let Some(current) = config.find_one(doc! {"_id":"access"}).await? {
        if PasswordHash::new(current.get_str("password_hash")?)
            .ok()
            .is_some_and(|hash| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &hash)
                    .is_ok()
            })
        {
            return Ok(());
        }
    }
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|_| "Création du sel impossible.")?;
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| "Création du secret administrateur impossible.")?
        .to_string();
    config
        .update_one(doc! {"_id":"access"}, doc! {"$set":{"password_hash":hash}})
        .upsert(true)
        .await?;
    database
        .collection::<Document>("admin_sessions")
        .delete_many(doc! {})
        .await?;
    Ok(())
}

fn token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("qomune_admin=").map(str::to_owned))
}
fn admin_cookie(value: &str, age: u32) -> HeaderValue {
    let secure = if std::env::var("COOKIE_SECURE").as_deref() == Ok("true") {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "qomune_admin={value}; Path=/; HttpOnly; SameSite=Strict; Max-Age={age}{secure}"
    ))
    .unwrap()
}
fn denied() -> ApiError {
    ApiError(
        StatusCode::UNAUTHORIZED,
        "Connectez-vous à l’administration.".into(),
    )
}
async fn authorize(database: &Database, headers: &HeaderMap) -> ApiResult<()> {
    let token = token(headers).ok_or_else(denied)?;
    let found = database
        .collection::<Document>("admin_sessions")
        .find_one(doc! {"_id":token,"expires_at":{"$gt":DateTime::now()}})
        .await?;
    if found.is_none() {
        return Err(denied());
    }
    Ok(())
}
#[derive(Deserialize)]
pub struct Credentials {
    password: String,
}
pub async fn login(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<Credentials>,
) -> ApiResult<Response> {
    if input.password.len() > 200 {
        return Err(denied());
    }
    let record = database
        .collection::<Document>("administration")
        .find_one(doc! {"_id":"access"})
        .await?;
    let Some(record) = record else {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "L’administration n’est pas encore configurée sur ce serveur.".into(),
        ));
    };
    let hash = record.get_str("password_hash").unwrap_or("").to_owned();
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).ok().is_some_and(|hash| {
            Argon2::default()
                .verify_password(input.password.as_bytes(), &hash)
                .is_ok()
        })
    })
    .await
    .unwrap_or(false);
    if !valid {
        return Err(denied());
    }
    let value = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    database
        .collection::<Document>("admin_sessions")
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
    database.collection::<Document>("admin_sessions").insert_one(doc! {"_id":&value,"expires_at":DateTime::from_millis((Utc::now()+Duration::hours(2)).timestamp_millis())}).await?;
    if let Some(old) = token(&headers) {
        database
            .collection::<Document>("admin_sessions")
            .delete_one(doc! {"_id":old})
            .await?;
    }
    let mut response = Json(json!({"ok":true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, admin_cookie(&value, 7200));
    Ok(response)
}
pub async fn logout(State(database): State<Database>, headers: HeaderMap) -> ApiResult<Response> {
    if let Some(value) = token(&headers) {
        database
            .collection::<Document>("admin_sessions")
            .delete_one(doc! {"_id":value})
            .await?;
    }
    let mut response = Json(json!({"ok":true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, admin_cookie("", 0));
    Ok(response)
}
#[derive(Deserialize)]
pub struct Filters {
    days: Option<i64>,
}
async fn count(database: &Database, collection: &str, filter: Document) -> ApiResult<u64> {
    Ok(database
        .collection::<Document>(collection)
        .count_documents(filter)
        .await?)
}
async fn group(
    database: &Database,
    collection: &str,
    pipeline: Vec<Document>,
) -> ApiResult<Vec<Document>> {
    Ok(database
        .collection::<Document>(collection)
        .aggregate(pipeline)
        .await?
        .try_collect()
        .await?)
}
fn number(row: &Document, field: &str) -> i64 {
    row.get_i64(field)
        .or_else(|_| row.get_i32(field).map(i64::from))
        .unwrap_or(0)
}

/// Les graphiques portent sur les publications datées, et les compteurs sur l'état actuel de la base.
pub async fn statistics(
    State(database): State<Database>,
    headers: HeaderMap,
    Query(filters): Query<Filters>,
) -> ApiResult<Json<Value>> {
    authorize(&database, &headers).await?;
    let days = filters.days.unwrap_or(30);
    if ![7, 30, 90].contains(&days) {
        return Err(invalid("Choisissez une période de 7, 30 ou 90 jours."));
    }
    let today = Utc::now().date_naive();
    let start = today - Duration::days(days - 1);
    let end = today + Duration::days(1);
    let date_filter =
        doc! {"published_at":{"$gte":format!("{start}T00:00:00"),"$lt":format!("{end}T00:00:00")}};
    let daily = group(&database,"publications",vec![doc! {"$match":date_filter.clone()},doc! {"$group":{"_id":{"date":{"$substrBytes":["$published_at",0,10]},"visibility":"$visibility"},"count":{"$sum":1}}}]).await?;
    let mut timeline: BTreeMap<String, Value> = (0..days)
        .map(|n| {
            let date = (start + Duration::days(n)).to_string();
            (date.clone(), json!({"date":date,"public":0,"private":0}))
        })
        .collect();
    for row in daily {
        if let Ok(id) = row.get_document("_id") {
            if let (Ok(date), Ok(visibility)) = (id.get_str("date"), id.get_str("visibility")) {
                if ["public", "private"].contains(&visibility) {
                    if let Some(day) = timeline.get_mut(date) {
                        day[visibility] = json!(number(&row, "count"));
                    }
                }
            }
        }
    }
    let categories = group(
        &database,
        "publications",
        vec![
            doc! {"$match":date_filter.clone()},
            doc! {"$group":{"_id":"$category","count":{"$sum":1}}},
            doc! {"$sort":{"count":-1,"_id":1}},
        ],
    )
    .await?;
    let top = group(&database,"publications",vec![doc! {"$match":date_filter},doc! {"$group":{"_id":"$commune_id","count":{"$sum":1}}},doc! {"$sort":{"count":-1,"_id":1}},doc! {"$limit":5},doc! {"$lookup":{"from":"communes","localField":"_id","foreignField":"id","as":"commune"}}]).await?;
    let active = group(
        &database,
        "members",
        vec![
            doc! {"$group":{"_id":"$commune_id"}},
            doc! {"$count":"count"},
        ],
    )
    .await?;
    Ok(Json(json!({
        "days":days,"generated_at":Utc::now().to_rfc3339(),
        "totals":{
            "communes":count(&database,"communes",doc! {}).await?,
            "registered_communes":active.first().map(|r|number(r,"count")).unwrap_or(0),
            "members":count(&database,"members",doc! {}).await?,
            "mayors":count(&database,"members",doc! {"role":"maire"}).await?,
            "public":count(&database,"publications",doc! {"visibility":"public"}).await?,
            "private":count(&database,"publications",doc! {"visibility":"private"}).await?,
            "comments":count(&database,"comments",doc! {}).await?,
            "reactions":count(&database,"reactions",doc! {}).await?
        },
        "timeline":timeline.into_values().collect::<Vec<_>>(),
        "categories":categories.iter().map(|r|json!({"name":r.get_str("_id").unwrap_or("Autres"),"count":number(r,"count")})).collect::<Vec<_>>(),
        "top_communes":top.iter().map(|r| { let name = r.get_array("commune").ok().and_then(|a|a.first()).and_then(|v|v.as_document()).and_then(|d|d.get_str("name").ok()).unwrap_or("Commune non référencée"); json!({"name":name,"count":number(r,"count")}) }).collect::<Vec<_>>()
    })))
}
