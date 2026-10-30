//! Comptes fictifs du Gouvernement et de la Présidence, isolés des élus municipaux et de l'administration.
use super::*;

const DEMO_PASSWORD: &str = "gouvernement-demo";
const COOKIE_NAME: &str = "qomune_government_demo";

#[derive(Deserialize)]
struct Directory {
    checked_at: String,
    members: Vec<DirectoryMember>,
}
#[derive(Deserialize)]
struct Presidency {
    checked_at: String,
    member: DirectoryMember,
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
    #[serde(default)]
    institution: String,
}
#[derive(Deserialize)]
struct StoredAccount {
    #[serde(flatten)]
    account: DemoAccount,
    password_hash: String,
}

/// Création idempotente : les redémarrages ne dupliquent pas les comptes fictifs.
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
                        "is_demo": true, "source_checked_at": &directory.checked_at, "institution":"gouvernement"
                    },
                    "$setOnInsert": {"id":id, "password_hash":&hash}
                },
            )
            .upsert(true)
            .await?;
    }
    // La Présidence reste hors de la liste des 35 membres du Gouvernement.
    // Son compte est fictif et possède un mot de passe public distinct.
    let presidency: Presidency = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../frontend/reference/presidency.json"
    )))?;
    let person = presidency.member;
    let id = format!("demo-presidence-{}", person.slug);
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|_| "Création du sel de démonstration impossible.")?;
    let hash = Argon2::default()
        .hash_password(b"presidence-demo", &salt)
        .map_err(|_| "Création du secret de démonstration impossible.")?
        .to_string();
    accounts.update_one(doc! {"id":&id},doc! {
        "$set": {
            "slug":&person.slug,"name":person.name,"function":person.role,"ministries":person.ministries,
            "email":format!("{}@presidence.demo.qomune.fr",person.slug),"institution":"presidence",
            "is_demo":true,"source_checked_at":presidency.checked_at
        },
        "$setOnInsert":{"id":id,"password_hash":hash}
    }).upsert(true).await?;
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
        "Connectez-vous à un compte fictif du Gouvernement ou de la Présidence.".into(),
    )
}
pub async fn me(State(database): State<Database>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({"account":authenticated(&database, &headers).await?}),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GovernmentPublication {
    title: String,
    body: String,
    category: String,
    request_id: String,
    commune_id: Option<String>,
}

/// Une diffusion nationale correspond à un seul document, sans 34 875 copies.
/// La signature, la visibilité publique et les mentions fictives sont imposées par le serveur.
pub async fn publish(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<GovernmentPublication>,
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
        return Err(invalid("Catégorie de diffusion inconnue."));
    }
    let request_id = Uuid::parse_str(&input.request_id)
        .map_err(|_| invalid("Identifiant de diffusion invalide."))?;
    // Une destination communale doit provenir du référentiel officiel chargé en base.
    // Sans code, la diffusion reste nationale et rétrocompatible avec le premier formulaire.
    let target = match input.commune_id {
        Some(code) => {
            if code.len() != 5 || !code.bytes().all(|c| c.is_ascii_alphanumeric()) {
                return Err(invalid("Choisissez une commune valide."));
            }
            Some(
                database
                    .collection::<Commune>("communes")
                    .find_one(doc! {"id":code})
                    .await?
                    .ok_or_else(|| {
                        invalid("La commune choisie n’existe pas dans le référentiel.")
                    })?,
            )
        }
        None => None,
    };
    let scope = if target.is_some() {
        "commune"
    } else {
        "national"
    };
    let commune_id = target
        .as_ref()
        .map(|commune| commune.id.as_str())
        .unwrap_or("");
    let commune_name = target.as_ref().map(|commune| commune.name.as_str());
    let presidency = author.institution == "presidence";
    let id = format!(
        "{}-{}-{}",
        if presidency { "presidence" } else { "national" },
        author.slug,
        request_id
    );
    let author_profile = if presidency {
        "/presidence".to_owned()
    } else {
        format!("/gouvernement/{}", author.slug)
    };
    let record = doc! {
        "id":&id, "author_id":&author.id, "commune_id":commune_id, "commune_name":commune_name, "scope":scope, "is_demo":true,
        "title":format!("Démonstration — {}",input.title.trim()),
        "body":format!("Publication fictive de démonstration. Ne constitue pas une communication officielle.\n\n{}",input.body.trim()),
        "category":input.category, "visibility":"public",
        "author_name":format!("{} (compte fictif)",author.name), "author_role":author.function,
        "author_profile":author_profile,"author_institution":&author.institution, "published_at":Utc::now().to_rfc3339()
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
    // Le reçu reflète le premier envoi, même si un client réutilise son identifiant de demande.
    let saved = collection
        .find_one(doc! {"id":&id})
        .await?
        .ok_or_else(not_found)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id":id, "scope":saved.get_str("scope").unwrap_or("national"),
            "commune_id":saved.get_str("commune_id").unwrap_or(""),
            "commune_name":saved.get_str("commune_name").ok(), "is_demo":true
        })),
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
        .find(doc! {"author_id":author.id,"scope":{"$in":["national","commune"]},"is_demo":true,"visibility":"public"})
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
