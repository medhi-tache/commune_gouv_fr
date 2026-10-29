//! API Rust / MongoDB du prototype Qomune.
//! Les comptes de démonstration ne constituent pas une vérification de mandat.
mod fixtures;
mod reference;
#[cfg(test)]
mod tests;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{Duration, Utc};
use futures_util::TryStreamExt;
use mongodb::{
    Client, Database, IndexModel,
    bson::{DateTime, Document, doc},
    options::{ClientOptions, IndexOptions},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

type ApiResult<T> = Result<T, ApiError>;

/// Les détails internes restent côté serveur et ne sont pas exposés dans les réponses publiques.
#[derive(Debug)]
struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<mongodb::error::Error> for ApiError {
    fn from(_error: mongodb::error::Error) -> Self {
        // Ne pas journaliser une URI MongoDB susceptible de contenir des identifiants.
        eprintln!("Une opération MongoDB a échoué.");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "La base de données est temporairement indisponible.".into(),
        )
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, message.into())
}
fn unauthorized() -> ApiError {
    ApiError(
        StatusCode::UNAUTHORIZED,
        "Connectez-vous à votre espace municipal.".into(),
    )
}
fn not_found() -> ApiError {
    ApiError(StatusCode::NOT_FOUND, "Publication introuvable.".into())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Commune {
    id: String,
    name: String,
    department: String,
    region: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Member {
    id: String,
    name: String,
    role: String,
    commune_id: String,
    email: String,
}
/// Le hash du mot de passe est séparé du modèle public renvoyé par /auth/me.
#[derive(Serialize, Deserialize)]
struct StoredMember {
    #[serde(flatten)]
    member: Member,
    password_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_by: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Session {
    #[serde(rename = "_id")]
    token: String,
    member_id: String,
    expires_at: DateTime,
}
#[derive(Serialize, Deserialize, Clone)]
struct Publication {
    id: String,
    commune_id: String,
    title: String,
    body: String,
    category: String,
    visibility: String,
    author_name: String,
    author_role: String,
    published_at: String,
}
/// L'identifiant d'auteur reste en base ; la signature conserve son état au moment de l'envoi.
#[derive(Serialize, Deserialize)]
struct StoredPublication {
    #[serde(flatten)]
    publication: Publication,
    author_id: String,
}

/// Le client MongoDB partage ses connexions de façon asynchrone entre les requêtes.
async fn connect(uri: &str, name: &str) -> mongodb::error::Result<Database> {
    let mut options = ClientOptions::parse(uri).await?;
    options.server_selection_timeout = Some(std::time::Duration::from_secs(5));
    let database = Client::with_options(options)?.database(name);
    database.run_command(doc! {"ping":1}).await?;
    Ok(database)
}

/// Les ajouts initiaux sont idempotents : un redémarrage ne remplace jamais les données existantes.
async fn initialize(database: &Database, password: &str) -> mongodb::error::Result<()> {
    for name in ["communes", "members", "publications"] {
        let index = IndexModel::builder()
            .keys(doc! {"id":1})
            .options(IndexOptions::builder().unique(true).build())
            .build();
        database
            .collection::<Document>(name)
            .create_index(index)
            .await?;
    }
    database
        .collection::<Document>("members")
        .create_index(
            IndexModel::builder()
                .keys(doc! {"email":1})
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    database
        .collection::<Document>("publications")
        .create_index(
            IndexModel::builder()
                .keys(doc! {"commune_id":1,"visibility":1,"published_at":-1})
                .build(),
        )
        .await?;
    // L'index TTL nettoie les sessions. Leur date est aussi contrôlée à chaque lecture.
    database
        .collection::<Document>("sessions")
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
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes()).expect("Sel valide");
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Hachage du mot de passe")
        .to_string();
    for (id, name, department, region, slug, author) in fixtures::COMMUNES {
        database
            .collection::<Document>("communes")
            .update_one(
                doc! {"id":id},
                doc! {"$setOnInsert":{"id":id,"name":name,"department":department,"region":region,"search_name":reference::normalize_name(name)}},
            )
            .upsert(true)
            .await?;
        let member_id = format!("maire-{id}");
        database.collection::<Document>("members").update_one(doc!{"id":&member_id},doc!{"$setOnInsert":{"id":&member_id,"name":author,"role":"maire","commune_id":id,"email":format!("maire.{slug}@demo.qomune.fr"),"password_hash":&hash}}).upsert(true).await?;
    }
    database.collection::<Document>("members").update_one(doc!{"id":"conseiller-74010"},doc!{"$setOnInsert":{"id":"conseiller-74010","name":"Léa Robert","role":"conseiller","commune_id":"74010","email":"conseil.annecy@demo.qomune.fr","password_hash":&hash}}).upsert(true).await?;
    for (index, (commune, title, body, category, visibility)) in
        fixtures::EXAMPLES.iter().enumerate()
    {
        let id = format!("demo-{}", index + 1);
        let author = database
            .collection::<StoredMember>("members")
            .find_one(doc! {"id":format!("maire-{commune}")})
            .await?
            .expect("Compte de démonstration initialisé")
            .member;
        database.collection::<Document>("publications").update_one(doc!{"id":&id},doc!{"$setOnInsert":{
            "id":id,"commune_id":commune.to_string(),"author_id":author.id,"title":title.to_string(),"body":body.to_string(),"category":category.to_string(),"visibility":visibility.to_string(),"author_name":author.name,"author_role":author.role,"published_at":(Utc::now()-Duration::hours(index as i64*17+2)).to_rfc3339()
        }}).upsert(true).await?;
    }
    Ok(())
}

fn session_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("qomune_session="))
}
fn cookie(value: &str, max_age: u32) -> HeaderValue {
    let secure = if std::env::var("COOKIE_SECURE").as_deref() == Ok("true") {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "qomune_session={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={max_age}{secure}"
    ))
    .expect("Cookie de session valide")
}
/// Les droits proviennent exclusivement du compte associé à une session valide en base.
async fn member(database: &Database, headers: &HeaderMap) -> ApiResult<Option<Member>> {
    let Some(token) = session_token(headers) else {
        return Ok(None);
    };
    let session = database
        .collection::<Session>("sessions")
        .find_one(doc! {"_id":token,"expires_at":{"$gt":DateTime::now()}})
        .await?;
    let Some(session) = session else {
        return Ok(None);
    };
    Ok(database
        .collection::<StoredMember>("members")
        .find_one(doc! {"id":session.member_id,"role":{"$in":["maire","conseiller"]}})
        .await?
        .map(|record| record.member))
}

async fn health(State(database): State<Database>) -> ApiResult<Json<Value>> {
    database.run_command(doc! {"ping":1}).await.map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "MongoDB est indisponible.".into(),
        )
    })?;
    Ok(Json(json!({"status":"ok","database":"mongodb"})))
}
#[derive(Deserialize, Default)]
struct CommuneFilters {
    q: Option<String>,
    ids: Option<String>,
    limit: Option<i64>,
}

/// Limite les réponses pour ne pas envoyer 34 875 options au navigateur. Les codes restent des chaînes.
async fn communes(
    State(database): State<Database>,
    Query(query): Query<CommuneFilters>,
) -> ApiResult<Json<Vec<Commune>>> {
    let mut filter = doc! {};
    let mut limit = query.limit.unwrap_or(20);
    if !(1..=100).contains(&limit) {
        return Err(invalid("La limite doit être comprise entre 1 et 100."));
    }
    if let Some(ids) = query.ids {
        let codes: Vec<_> = ids.split(',').filter(|id| !id.is_empty()).collect();
        if codes.is_empty()
            || codes.len() > 200
            || codes
                .iter()
                .any(|id| id.len() != 5 || !id.bytes().all(|c| c.is_ascii_alphanumeric()))
        {
            return Err(invalid("Liste de codes de communes invalide."));
        }
        limit = codes.len() as i64;
        filter.insert("id", doc! {"$in":codes});
    }
    if let Some(q) = query.q.filter(|q| !q.trim().is_empty()) {
        if q.len() > 200 {
            return Err(invalid("Recherche de commune trop longue."));
        }
        let q = reference::normalize_name(&q);
        if q.is_empty() {
            return Ok(Json(Vec::new()));
        }
        let literal = escaped_search(&q);
        filter.insert(
            "$or",
            vec![
                doc! {"search_name":{"$regex":&literal}},
                doc! {"id":{"$regex":format!("^{literal}"),"$options":"i"}},
            ],
        );
    }
    Ok(Json(
        database
            .collection::<Commune>("communes")
            .find(filter)
            .sort(doc! {"search_name":1,"id":1})
            .limit(limit)
            .await?
            .try_collect()
            .await?,
    ))
}

async fn communes_meta(State(database): State<Database>) -> ApiResult<Json<Value>> {
    let total = database
        .collection::<Document>("communes")
        .count_documents(doc! {})
        .await?;
    let source = database
        .collection::<Document>("reference_imports")
        .find_one(doc! {"_id":"communes"})
        .await?;
    Ok(Json(
        json!({"total":total,"vintage":source.as_ref().and_then(|d|d.get_i32("vintage").ok()),"source_url":source.as_ref().and_then(|d|d.get_str("source_url").ok()),"imported_at":source.as_ref().and_then(|d|d.get_str("imported_at").ok())}),
    ))
}
#[derive(Deserialize)]
struct Login {
    email: String,
    password: String,
}
async fn login(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<Login>,
) -> ApiResult<Response> {
    if input.email.len() > 200 || input.password.len() > 200 {
        return Err(invalid("Identifiants trop longs."));
    }
    let record = database
        .collection::<StoredMember>("members")
        .find_one(doc! {"email":input.email.trim().to_lowercase()})
        .await?;
    let Some(record) = record else {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Identifiants incorrects.".into(),
        ));
    };
    let id = record.member.id;
    // Argon2 est volontairement coûteux : son calcul est isolé des tâches réseau du serveur.
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&record.password_hash)
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
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "Identifiants incorrects.".into(),
        ));
    }
    if let Some(old) = session_token(&headers) {
        database
            .collection::<Session>("sessions")
            .delete_one(doc! {"_id":old})
            .await?;
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    database
        .collection::<Session>("sessions")
        .insert_one(Session {
            token: token.clone(),
            member_id: id,
            expires_at: DateTime::from_millis((Utc::now() + Duration::hours(8)).timestamp_millis()),
        })
        .await?;
    let mut response = Json(json!({"ok":true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie(&token, 28800));
    Ok(response)
}
async fn logout(State(database): State<Database>, headers: HeaderMap) -> ApiResult<Response> {
    if let Some(token) = session_token(&headers) {
        database
            .collection::<Session>("sessions")
            .delete_one(doc! {"_id":token})
            .await?;
    }
    let mut response = Json(json!({"ok":true})).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie("", 0));
    Ok(response)
}
async fn me(State(database): State<Database>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    Ok(Json(json!({"member":member(&database,&headers).await?})))
}

/// Cette autorisation est vérifiée par l'API, indépendamment des boutons affichés dans Next.js.
async fn require_mayor(database: &Database, headers: &HeaderMap) -> ApiResult<Member> {
    let user = member(database, headers).await?.ok_or_else(unauthorized)?;
    if user.role != "maire" {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Seul le maire peut gérer les membres de son conseil municipal.".into(),
        ));
    }
    Ok(user)
}

/// Les coordonnées du conseil ne sont visibles que par son maire. Aucun hash n'est renvoyé.
async fn council_members(
    State(database): State<Database>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Member>>> {
    let mayor = require_mayor(&database, &headers).await?;
    let records: Vec<StoredMember> = database
        .collection("members")
        .find(doc! {"commune_id":mayor.commune_id})
        .sort(doc! {"role":-1,"name":1})
        .await?
        .try_collect()
        .await?;
    Ok(Json(
        records.into_iter().map(|record| record.member).collect(),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewMember {
    name: String,
    email: String,
}

/// Validation des adresses usuelles du prototype ; les noms de domaine internationaux doivent être en ASCII.
fn valid_email(email: &str) -> bool {
    if email.len() > 200 || !email.is_ascii() {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._%+-".contains(&c))
        && domain.contains('.')
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
}

async fn create_member(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<NewMember>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let mayor = require_mayor(&database, &headers).await?;
    let name = input.name.trim();
    let email = input.email.trim().to_lowercase();
    if !(2..=120).contains(&name.chars().count()) || name.chars().any(char::is_control) {
        return Err(invalid(
            "Le nom doit contenir de 2 à 120 caractères, sans caractères de contrôle.",
        ));
    }
    if !valid_email(&email) {
        return Err(invalid(
            "Saisissez une adresse e-mail valide (200 caractères maximum).",
        ));
    }
    // Le mot de passe aléatoire n'est renvoyé qu'à la création ; seule son empreinte est stockée.
    let initial_password = format!("Qm-{}", Uuid::new_v4().simple());
    let password_to_hash = initial_password.clone();
    let hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes()).expect("Sel valide");
        Argon2::default()
            .hash_password(password_to_hash.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    })
    .await
    .map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "La création du compte a échoué.".into(),
        )
    })?
    .map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "La création du compte a échoué.".into(),
        )
    })?;
    let new_member = Member {
        id: Uuid::new_v4().to_string(),
        name: name.into(),
        email,
        role: "conseiller".into(),
        commune_id: mayor.commune_id,
    };
    let record = StoredMember {
        member: new_member.clone(),
        password_hash: hash,
        created_by: Some(mayor.id),
    };
    if let Err(error) = database
        .collection::<StoredMember>("members")
        .insert_one(record)
        .await
    {
        // L'index unique bloque également deux créations simultanées avec la même adresse.
        if matches!(error.kind.as_ref(), mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(write)) if write.code==11000)
        {
            return Err(ApiError(
                StatusCode::CONFLICT,
                "Cette adresse e-mail est déjà utilisée. Aucun compte existant n'a été modifié."
                    .into(),
            ));
        }
        return Err(error.into());
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({"member":new_member,"initial_password":initial_password})),
    ))
}
#[derive(Deserialize, Default)]
struct Filters {
    commune: Option<String>,
    q: Option<String>,
    category: Option<String>,
    scope: Option<String>,
}
/// La recherche est littérale : les caractères spéciaux ne deviennent pas des opérateurs de regex.
fn escaped_search(query: &str) -> String {
    query.chars().fold(String::new(), |mut result, character| {
        if ".*+?^${}()|[]\\".contains(character) {
            result.push('\\');
        }
        result.push(character);
        result
    })
}
async fn publications(
    State(database): State<Database>,
    headers: HeaderMap,
    Query(filters): Query<Filters>,
) -> ApiResult<Json<Vec<Publication>>> {
    let mut filter = doc! {};
    let scope = filters.scope.as_deref().unwrap_or("public");
    match scope {
        "public" => {
            filter.insert("visibility", "public");
            if let Some(commune) = filters.commune.filter(|c| !c.is_empty()) {
                filter.insert("commune_id", commune);
            }
        }
        "council" => {
            let user = member(&database, &headers)
                .await?
                .ok_or_else(unauthorized)?;
            if filters
                .commune
                .as_ref()
                .is_some_and(|id| !id.is_empty() && id != &user.commune_id)
            {
                return Err(ApiError(
                    StatusCode::FORBIDDEN,
                    "Cette commune ne correspond pas à votre mandat.".into(),
                ));
            }
            filter.insert("commune_id", user.commune_id);
            filter.insert("visibility", doc! {"$in":["public","private"]});
        }
        _ => return Err(invalid("Vue inconnue.")),
    }
    if let Some(category) = filters.category.filter(|c| !c.is_empty()) {
        filter.insert("category", category);
    }
    if let Some(query) = filters.q.filter(|q| !q.is_empty()) {
        if query.len() > 200 {
            return Err(invalid("Recherche trop longue."));
        }
        let literal = escaped_search(&query);
        filter.insert(
            "$or",
            vec![
                doc! {"title":{"$regex":&literal,"$options":"i"}},
                doc! {"body":{"$regex":&literal,"$options":"i"}},
            ],
        );
    }
    // MongoDB applique la visibilité avant de renvoyer les documents, y compris pour une recherche.
    let records: Vec<StoredPublication> = database
        .collection("publications")
        .find(filter)
        .sort(doc! {"published_at":-1})
        .limit(100)
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
async fn publication(
    State(database): State<Database>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<Publication>> {
    let mut filter = doc! {"id":id,"visibility":"public"};
    if let Some(user) = member(&database, &headers).await? {
        filter.remove("visibility");
        filter.insert(
            "$or",
            vec![
                doc! {"visibility":"public"},
                doc! {"visibility":"private","commune_id":user.commune_id},
            ],
        );
    }
    let record = database
        .collection::<StoredPublication>("publications")
        .find_one(filter)
        .await?
        .ok_or_else(not_found)?;
    Ok(Json(record.publication))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewPublication {
    title: String,
    body: String,
    category: String,
    visibility: String,
}
async fn create_publication(
    State(database): State<Database>,
    headers: HeaderMap,
    Json(input): Json<NewPublication>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let user = member(&database, &headers)
        .await?
        .ok_or_else(unauthorized)?;
    if !(3..=160).contains(&input.title.trim().chars().count())
        || !(10..=10000).contains(&input.body.trim().chars().count())
    {
        return Err(invalid(
            "Le titre doit contenir 3 à 160 caractères et le texte 10 à 10 000 caractères.",
        ));
    }
    if !["public", "private"].contains(&input.visibility.as_str()) {
        return Err(invalid("Visibilité inconnue."));
    }
    if ![
        "Vie citoyenne",
        "Travaux",
        "Culture",
        "Environnement",
        "Services publics",
        "Conseil municipal",
    ]
    .contains(&input.category.as_str())
    {
        return Err(invalid("Catégorie inconnue."));
    }
    let id = Uuid::new_v4().to_string();
    let record = StoredPublication {
        author_id: user.id,
        publication: Publication {
            id: id.clone(),
            commune_id: user.commune_id,
            title: input.title.trim().into(),
            body: input.body.trim().into(),
            category: input.category,
            visibility: input.visibility,
            author_name: user.name,
            author_role: user.role,
            published_at: Utc::now().to_rfc3339(),
        },
    };
    database
        .collection::<StoredPublication>("publications")
        .insert_one(record)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({"id":id}))))
}
fn app(database: Database) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/communes", get(communes))
        .route("/communes/meta", get(communes_meta))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/members", get(council_members).post(create_member))
        .route("/publications", get(publications).post(create_publication))
        .route("/publications/{id}", get(publication))
        .layer(DefaultBodyLimit::max(65536))
        .with_state(database)
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".into());
    let name = std::env::var("MONGODB_DATABASE").unwrap_or_else(|_| "qomune_prototype".into());
    let database = connect(&uri, &name)
        .await
        .map_err(|_| "Connexion à MongoDB impossible. Démarrez MongoDB et vérifiez MONGODB_URI.")?;
    let password = std::env::var("QOMUNE_DEMO_PASSWORD").unwrap_or_else(|_| "commune-demo".into());
    initialize(&database, &password)
        .await
        .map_err(|_| "Initialisation MongoDB impossible.")?;
    let count = reference::import(&database)
        .await
        .map_err(|_| "Import du référentiel des communes impossible.")?;
    println!("{count} communes COG 2026 chargées depuis le référentiel data.gouv.fr / Insee.");
    if std::env::args().any(|arg| arg == "--import-communes") {
        return Ok(());
    }
    let address = std::env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("API Qomune : http://{address} — Rust / MongoDB — publications de démonstration");
    axum::serve(listener, app(database))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
