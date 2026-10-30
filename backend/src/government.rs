//! Comptes de démonstration du Gouvernement, isolés des élus municipaux et de l'administration.
use super::*;

const DEMO_PASSWORD: &str = "gouvernement-demo";
const COOKIE_NAME: &str = "qomune_government_demo";

#[derive(Deserialize)]
struct Directory {
    checked_at: String,
    members: Vec<DirectoryMember>,
}
#[derive(Deserialize)]
struct DirectoryMember {
    slug: String,
    name: String,
    role: String,
    ministries: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct DemoAccount {
    id: String,
    slug: String,
    name: String,
    function: String,
    ministries: Vec<String>,
    email: String,
    is_demo: bool,
    source_checked_at: String,
}
#[derive(Deserialize)]
struct StoredAccount {
    #[serde(flatten)]
    account: DemoAccount,
    password_hash: String,
}

/// Création idempotente : les redémarrages ne dupliquent pas les 35 comptes.
/// Le référentiel est commun au frontend et au binaire Rust.
pub async fn initialize(database: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let directory: Directory = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../frontend/reference/government.json"
    )))?;
    let accounts = database.collection::<Document>("government_demo_accounts");
    for key in ["id", "email"] {
        accounts
            .create_index(
                IndexModel::builder()
                    .keys(doc! {key:1})
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;
    }
    database
        .collection::<Document>("government_demo_sessions")
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
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|_| "Création du sel de démonstration impossible.")?;
    let hash = Argon2::default()
        .hash_password(DEMO_PASSWORD.as_bytes(), &salt)
        .map_err(|_| "Création du secret de démonstration impossible.")?
        .to_string();
    for person in directory.members {
        let id = format!("demo-gouvernement-{}", person.slug);
        accounts
            .update_one(
                doc! {"id":&id},
                doc! {
                    "$set": {
                        "slug": &person.slug, "name": person.name, "function": person.role,
                        "ministries": person.ministries,
                        "email": format!("{}@gouvernement.demo.qomune.fr", person.slug),
                        "is_demo": true, "source_checked_at": &directory.checked_at
                    },
                    "$setOnInsert": {"id":id, "password_hash":&hash}
                },
            )
            .upsert(true)
            .await?;
    }
    Ok(())
}

fn token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("qomune_government_demo="))
}
fn session_cookie(value: &str, age: u32) -> HeaderValue {
    let secure = if std::env::var("COOKIE_SECURE").as_deref() == Ok("true") {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "{COOKIE_NAME}={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={age}{secure}"
    ))
    .unwrap()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credentials {
    email: String,
    password: String,
}

pub async fn login(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<Credentials>,
) -> ApiResult<Response> {
    let denied = || {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "Identifiants de démonstration incorrects.".into(),
        )
    };
    if input.email.len() > 200 || input.password.len() > 200 {
        return Err(denied());
    }
    let account = database
        .collection::<StoredAccount>("government_demo_accounts")
        .find_one(doc! {"email":input.email.trim().to_lowercase(),"is_demo":true})
        .await?
        .ok_or_else(denied)?;
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&account.password_hash)
            .ok()
            .is_some_and(|hash| {
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
    if let Some(old) = token(&headers) {
        database
            .collection::<Session>("government_demo_sessions")
            .delete_one(doc! {"_id":old})
            .await?;
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    database
        .collection::<Session>("government_demo_sessions")
        .insert_one(Session {
            token: token.clone(),
            member_id: account.account.id,
            expires_at: DateTime::from_millis((Utc::now() + Duration::hours(2)).timestamp_millis()),
        })
        .await?;
    let mut response = Json(json!({"ok":true,"is_demo":true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, session_cookie(&token, 7200));
    Ok(response)
}

/// Ne reconnaît que le cookie et la collection de sessions du Gouvernement.
async fn authenticated(database: &Database, headers: &HeaderMap) -> ApiResult<Option<DemoAccount>> {
    let Some(token) = token(headers) else {
        return Ok(None);
    };
    let session = database
        .collection::<Session>("government_demo_sessions")
        .find_one(doc! {"_id":token,"expires_at":{"$gt":DateTime::now()}})
        .await?;
    let Some(session) = session else {
        return Ok(None);
    };
    Ok(database
        .collection::<StoredAccount>("government_demo_accounts")
        .find_one(doc! {"id":session.member_id,"is_demo":true})
        .await?
        .map(|record| record.account))
}
fn denied() -> ApiError {
    ApiError(
        StatusCode::UNAUTHORIZED,
        "Connectez-vous à un compte fictif du Gouvernement.".into(),
    )
}
pub async fn me(State(database): State<Database>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({"account":authenticated(&database, &headers).await?}),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NationalPublication {
    title: String,
    body: String,
    category: String,
    request_id: String,
}

/// Une diffusion nationale correspond à un seul document, sans 34 875 copies.
/// La signature, la visibilité publique et les mentions fictives sont imposées par le serveur.
pub async fn publish(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<NationalPublication>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let author = authenticated(&database, &headers)
        .await?
        .ok_or_else(denied)?;
    if !(3..=140).contains(&input.title.trim().chars().count())
        || !(10..=10000).contains(&input.body.trim().chars().count())
    {
        return Err(invalid(
            "Le titre doit contenir 3 à 140 caractères et le texte 10 à 10 000 caractères.",
        ));
    }
    if ![
        "Vie citoyenne",
        "Travaux",
        "Culture",
        "Environnement",
        "Services publics",
    ]
    .contains(&input.category.as_str())
    {
        return Err(invalid("Catégorie de diffusion nationale inconnue."));
    }
    let request_id = Uuid::parse_str(&input.request_id)
        .map_err(|_| invalid("Identifiant de diffusion invalide."))?;
    let id = format!("national-{}-{}", author.slug, request_id);
    let record = doc! {
        "id":&id, "author_id":&author.id, "commune_id":"", "scope":"national", "is_demo":true,
        "title":format!("Démonstration — {}",input.title.trim()),
        "body":format!("Publication fictive de démonstration. Ne constitue pas une communication officielle.\n\n{}",input.body.trim()),
        "category":input.category, "visibility":"public",
        "author_name":format!("{} (compte fictif)",author.name), "author_role":author.function,
        "author_profile":format!("/gouvernement/{}",author.slug), "published_at":Utc::now().to_rfc3339()
    };
    let collection = database.collection::<Document>("publications");
    // Un nouvel essai après une coupure réseau conserve le même identifiant, sans doublon.
    if let Err(error) = collection
        .update_one(doc! {"id":&id}, doc! {"$setOnInsert":record})
        .upsert(true)
        .await
    {
        if !matches!(error.kind.as_ref(),mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(write)) if write.code==11000)
        {
            return Err(error.into());
        }
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({"id":id,"scope":"national","is_demo":true})),
    ))
}

pub async fn publications(
    State(database): State<Database>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Publication>>> {
    let author = authenticated(&database, &headers)
        .await?
        .ok_or_else(denied)?;
    let records: Vec<StoredPublication> = database
        .collection("publications")
        .find(doc! {"author_id":author.id,"scope":"national","visibility":"public"})
        .sort(doc! {"published_at":-1,"id":-1})
        .limit(20)
        .await?
        .try_collect()
        .await?;
    Ok(Json(
        records
            .into_iter()
            .map(|record| record.publication)
            .collect(),
    ))
}

pub async fn logout(State(database): State<Database>, headers: HeaderMap) -> ApiResult<Response> {
    if let Some(token) = token(&headers) {
        database
            .collection::<Session>("government_demo_sessions")
            .delete_one(doc! {"_id":token})
            .await?;
    }
    let mut response = Json(json!({"ok":true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, session_cookie("", 0));
    Ok(response)
}
