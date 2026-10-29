//! Inscription déclarative des maires dans le prototype uniquement.
//! FranceConnect et la validation du mandat sont prévus pour la version de production.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registration {
    pub name: String,
    pub email: String,
    pub password: String,
    pub commune_id: String,
}

pub fn prototype_mode() -> bool {
    std::env::var("QOMUNE_MODE").unwrap_or_else(|_| "prototype".into()) == "prototype"
}

pub async fn register(
    State(database): State<Database>,
    Json(input): Json<Registration>,
) -> ApiResult<Response> {
    register_for_mode(database, input, prototype_mode()).await
}

/// Le paramètre explicite permet de tester le refus en production sans modifier l'environnement des autres tests.
pub async fn register_for_mode(
    database: Database,
    input: Registration,
    prototype: bool,
) -> ApiResult<Response> {
    if !prototype {
        return Err(ApiError(StatusCode::FORBIDDEN,"L'inscription sans vérification est réservée au prototype. FranceConnect est prévu pour la production.".into()));
    }
    let name = input.name.trim();
    let email = input.email.trim().to_lowercase();
    if !(2..=120).contains(&name.chars().count()) || name.chars().any(char::is_control) {
        return Err(invalid(
            "Le nom doit contenir de 2 à 120 caractères, sans caractères de contrôle.",
        ));
    }
    if !valid_email(&email) {
        return Err(invalid("Saisissez une adresse e-mail valide."));
    }
    if input.password.chars().count() < 12 || input.password.len() > 200 {
        return Err(invalid(
            "Le mot de passe doit contenir au moins 12 caractères et au plus 200 octets.",
        ));
    }
    if input.commune_id.len() != 5
        || database
            .collection::<Commune>("communes")
            .find_one(doc! {"id":&input.commune_id})
            .await?
            .is_none()
    {
        return Err(invalid("Choisissez une commune du référentiel officiel."));
    }
    let password = input.password.clone();
    let hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes()).expect("Sel valide");
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
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
    // L'inscription est déclarative : aucune identité ni aucun mandat ne sont présentés comme vérifiés.
    let record = doc! {"id":Uuid::new_v4().to_string(),"name":name,"email":&email,"role":"maire","commune_id":&input.commune_id,
    "password_hash":hash,"verification_status":"unverified_prototype","created_at":Utc::now().to_rfc3339()};
    if let Err(error) = database
        .collection::<Document>("members")
        .insert_one(record)
        .await
    {
        if matches!(error.kind.as_ref(),mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(write)) if write.code==11000)
        {
            return Err(ApiError(StatusCode::CONFLICT,"Un compte existe déjà pour cette adresse e-mail ou un maire est déjà inscrit pour cette commune.".into()));
        }
        return Err(error.into());
    }
    let mut response = login(
        State(database),
        HeaderMap::new(),
        Json(Login {
            email,
            password: input.password,
        }),
    )
    .await?;
    *response.status_mut() = StatusCode::CREATED;
    Ok(response)
}
