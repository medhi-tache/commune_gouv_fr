//! Tests d'intégration contre une vraie instance MongoDB.
//! Chaque test crée puis supprime une base qomune_test_<UUID>, sans toucher à la base de l'application.
use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use tower::ServiceExt;

async fn setup() -> (Router, Database) {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".into());
    let name = format!("qomune_test_{}", Uuid::new_v4().simple());
    let database = connect(&uri, &name)
        .await
        .expect("MongoDB doit être démarré pour les tests");
    initialize(&database, "commune-demo").await.unwrap();
    (app(database.clone()), database)
}
async fn request(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value, Option<String>) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header(header::COOKIE, token);
    }
    if body.is_some() {
        request = request.header(header::CONTENT_TYPE, "application/json");
    }
    let response = app
        .clone()
        .oneshot(
            request
                .body(Body::from(body.map(|b| b.to_string()).unwrap_or_default()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_string());
    let bytes = to_bytes(response.into_body(), 65536).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        cookie,
    )
}
async fn sign_in(app: &Router, email: &str) -> String {
    let (status, _, cookie) = request(
        app,
        "POST",
        "/auth/login",
        None,
        Some(json!({"email":email,"password":"commune-demo"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    cookie.unwrap()
}
#[tokio::test]
async fn visiteurs_sans_acces_aux_publications_privees() {
    let (app, database) = setup().await;
    let (status, items, _) = request(&app, "GET", "/publications", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(items.as_array().unwrap().len(), 7);
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["visibility"] == "public")
    );
    assert_eq!(
        request(&app, "GET", "/publications/demo-8", None, None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&app, "GET", "/publications?scope=council", None, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let (_, items, _) = request(
        &app,
        "GET",
        "/publications?q=Document%20de%20travail",
        None,
        None,
    )
    .await;
    assert!(items.as_array().unwrap().is_empty());
    database.drop().await.unwrap();
}
#[tokio::test]
async fn conseils_isoles_meme_par_lien_direct() {
    let (app, database) = setup().await;
    let token = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    assert_eq!(
        request(&app, "GET", "/publications/demo-8", Some(&token), None)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(&app, "GET", "/publications/demo-9", Some(&token), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/publications?scope=council&commune=33063",
            Some(&token),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (_, items, _) = request(
        &app,
        "GET",
        "/publications?scope=council",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(items.as_array().unwrap().len(), 3);
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["commune_id"] == "74010")
    );
    let (_, public, _) = request(&app, "GET", "/publications", Some(&token), None).await;
    assert!(
        public
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["visibility"] == "public")
    );
    let (_, session, _) = request(&app, "GET", "/auth/me", Some(&token), None).await;
    assert!(session["member"].get("password_hash").is_none());
    database.drop().await.unwrap();
}
#[tokio::test]
async fn conseillere_peut_publier_sans_usurper_un_mandat() {
    let (app, database) = setup().await;
    let token = sign_in(&app, "conseil.annecy@demo.qomune.fr").await;
    let content = json!({"title":"Une note pour notre conseil","body":"Une publication réservée aux membres du conseil.","category":"Conseil municipal","visibility":"private"});
    let (status, result, _) = request(
        &app,
        "POST",
        "/publications",
        Some(&token),
        Some(content.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let path = format!("/publications/{}", result["id"].as_str().unwrap());
    let (_, item, _) = request(&app, "GET", &path, Some(&token), None).await;
    assert_eq!(item["commune_id"], "74010");
    assert_eq!(item["author_role"], "conseiller");
    assert_eq!(
        request(&app, "GET", &path, None, None).await.0,
        StatusCode::NOT_FOUND
    );
    let other = sign_in(&app, "maire.bordeaux@demo.qomune.fr").await;
    assert_eq!(
        request(&app, "GET", &path, Some(&other), None).await.0,
        StatusCode::NOT_FOUND
    );
    let mut forged = content;
    forged["commune_id"] = json!("33063");
    assert_eq!(
        request(&app, "POST", "/publications", Some(&token), Some(forged))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        database
            .collection::<Document>("publications")
            .count_documents(doc! {"author_id":"conseiller-74010"})
            .await
            .unwrap(),
        1
    );
    database.drop().await.unwrap();
}
#[tokio::test]
async fn session_expiree_et_deconnexion_revoquent_les_droits() {
    let (app, database) = setup().await;
    let token = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    assert_eq!(
        request(&app, "POST", "/auth/logout", Some(&token), Some(json!({})))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/publications?scope=council",
            Some(&token),
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let token = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    database
        .collection::<Document>("sessions")
        .update_many(
            doc! {},
            doc! {"$set":{"expires_at":DateTime::from_millis(0)}},
        )
        .await
        .unwrap();
    assert_eq!(
        request(
            &app,
            "GET",
            "/publications?scope=council",
            Some(&token),
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    database.drop().await.unwrap();
}
#[tokio::test]
async fn validation_et_recherche_litterale() {
    let (app, database) = setup().await;
    let token = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let invalid = json!({"title":"x","body":"court","category":"Culture","visibility":"public"});
    assert_eq!(
        request(&app, "POST", "/publications", Some(&token), Some(invalid))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let valid = json!({"title":"Une information publique","body":"Cette information doit être visible pour tous.","category":"Culture","visibility":"public"});
    assert_eq!(
        request(&app, "POST", "/publications", None, Some(valid.clone()))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&app, "POST", "/publications", Some(&token), Some(valid))
            .await
            .0,
        StatusCode::CREATED
    );
    let (_, items, _) = request(
        &app,
        "GET",
        "/publications?commune=74010&category=Culture&q=INFORMATION",
        None,
        None,
    )
    .await;
    assert_eq!(items.as_array().unwrap().len(), 1);
    assert_eq!(items[0]["title"], "Une information publique");
    let (_, items, _) = request(&app, "GET", "/publications?q=.%2A", None, None).await;
    assert!(
        items.as_array().unwrap().is_empty(),
        "La saisie .* doit être cherchée littéralement"
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/auth/login",
            None,
            Some(json!({"email":"maire.annecy@demo.qomune.fr","password":"incorrect"}))
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    database.drop().await.unwrap();
}
#[tokio::test]
async fn donnees_conservees_apres_reconnexion_et_initialisation() {
    let (app, database) = setup().await;
    database
        .collection::<Document>("publications")
        .update_one(
            doc! {"id":"demo-1"},
            doc! {"$set":{"title":"Contenu conservé"}},
        )
        .await
        .unwrap();
    initialize(&database, "autre-mot-de-passe").await.unwrap();
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".into());
    let reconnected = connect(&uri, database.name()).await.unwrap();
    let stored = reconnected
        .collection::<Publication>("publications")
        .find_one(doc! {"id":"demo-1"})
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.title, "Contenu conservé");
    assert_eq!(
        reconnected
            .collection::<Document>("publications")
            .count_documents(doc! {})
            .await
            .unwrap(),
        9
    );
    // Le mot de passe initial reste valable : les comptes existants ne sont pas écrasés.
    sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    database.drop().await.unwrap();
}
