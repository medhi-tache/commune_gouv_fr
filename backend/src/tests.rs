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
async fn comptes_gouvernement_fictifs_connectables_et_isoles() {
    let (app, database) = setup().await;
    government::initialize(&database).await.unwrap();
    government::initialize(&database).await.unwrap();
    let accounts: Vec<Document> = database
        .collection("government_demo_accounts")
        .find(doc! {})
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(accounts.len(), 35, "Aucun doublon après un redémarrage");
    let mut last_cookie = String::new();
    for account in &accounts {
        assert!(account.get_bool("is_demo").unwrap());
        assert!(!account.get_str("function").unwrap().is_empty());
        assert!(
            account
                .get_str("password_hash")
                .unwrap()
                .starts_with("$argon2")
        );
        let (status, _, cookie) = request(
            &app,
            "POST",
            "/government-demo/login",
            None,
            Some(json!({"email":account.get_str("email").unwrap(),"password":"gouvernement-demo"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        last_cookie = cookie.unwrap();
        let (_, profile, _) =
            request(&app, "GET", "/government-demo/me", Some(&last_cookie), None).await;
        assert_eq!(profile["account"]["slug"], account.get_str("slug").unwrap());
        assert_eq!(profile["account"]["is_demo"], true);
        assert!(profile["account"].get("password_hash").is_none());
    }
    assert!(last_cookie.starts_with("qomune_government_demo="));
    assert!(
        request(&app, "GET", "/auth/me", Some(&last_cookie), None)
            .await
            .1["member"]
            .is_null()
    );
    for path in [
        "/publications?scope=council",
        "/members",
        "/admin/statistics",
    ] {
        assert_eq!(
            request(&app, "GET", path, Some(&last_cookie), None).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        request(
            &app,
            "POST",
            "/publications",
            Some(&last_cookie),
            Some(json!({
                "title":"Essai gouvernement", "body":"Une publication qui doit être refusée.",
                "category":"Vie citoyenne", "visibility":"public"
            }))
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    // Même en renommant son cookie, une session de démonstration ne devient pas municipale.
    let forged = last_cookie.replace("qomune_government_demo=", "qomune_session=");
    assert_eq!(
        request(&app, "GET", "/members", Some(&forged), None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/government-demo/login",
            None,
            Some(json!({
                "email":accounts[0].get_str("email").unwrap(), "password":"incorrect"
            }))
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    request(
        &app,
        "POST",
        "/government-demo/logout",
        Some(&last_cookie),
        Some(json!({})),
    )
    .await;
    assert!(
        request(&app, "GET", "/government-demo/me", Some(&last_cookie), None)
            .await
            .1["account"]
            .is_null()
    );
    let municipal = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    assert!(
        request(&app, "GET", "/government-demo/me", Some(&municipal), None)
            .await
            .1["account"]
            .is_null()
    );
    database.drop().await.unwrap();
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

#[tokio::test]
async fn gestion_des_membres_reservee_au_maire() {
    let (app, database) = setup().await;
    let payload = json!({"name":"Alex Dupont","email":"alex.dupont@example.test"});
    assert_eq!(
        request(&app, "GET", "/members", None, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&app, "POST", "/members", None, Some(payload.clone()))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let adviser = sign_in(&app, "conseil.annecy@demo.qomune.fr").await;
    assert_eq!(
        request(&app, "GET", "/members", Some(&adviser), None)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&app, "POST", "/members", Some(&adviser), Some(payload))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let (status, items, _) = request(&app, "GET", "/members", Some(&mayor), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(items.as_array().unwrap().len(), 2);
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["commune_id"] == "74010"
                && item.get("password_hash").is_none()
                && item.get("initial_password").is_none())
    );
    let bordeaux = sign_in(&app, "maire.bordeaux@demo.qomune.fr").await;
    let (_, items, _) = request(&app, "GET", "/members", Some(&bordeaux), None).await;
    assert_eq!(items.as_array().unwrap().len(), 1);
    assert_eq!(items[0]["commune_id"], "33063");
    database.drop().await.unwrap();
}

#[tokio::test]
async fn nouveau_membre_connecte_uniquement_a_son_conseil() {
    let (app, database) = setup().await;
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let (status, result, _) = request(
        &app,
        "POST",
        "/members",
        Some(&mayor),
        Some(json!({"name":"  Alex Dupont  ","email":" ALEX.DUPONT@example.test "})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(result["member"]["role"], "conseiller");
    assert_eq!(result["member"]["commune_id"], "74010");
    assert_eq!(result["member"]["name"], "Alex Dupont");
    let password = result["initial_password"].as_str().unwrap();
    assert!(password.len() >= 32);
    let stored = database
        .collection::<StoredMember>("members")
        .find_one(doc! {"email":"alex.dupont@example.test"})
        .await
        .unwrap()
        .unwrap();
    assert_ne!(stored.password_hash, password);
    assert_eq!(stored.created_by.as_deref(), Some("maire-74010"));
    let (status, _, token) = request(
        &app,
        "POST",
        "/auth/login",
        None,
        Some(json!({"email":"alex.dupont@example.test","password":password})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = token.unwrap();
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
        request(&app, "GET", "/members", Some(&token), None).await.0,
        StatusCode::FORBIDDEN
    );
    let publication = json!({"title":"Information du nouveau conseiller","body":"Une publication privée écrite par le nouveau membre.","category":"Conseil municipal","visibility":"private"});
    assert_eq!(
        request(
            &app,
            "POST",
            "/publications",
            Some(&token),
            Some(publication)
        )
        .await
        .0,
        StatusCode::CREATED
    );
    let (_, items, _) = request(&app, "GET", "/members", Some(&mayor), None).await;
    assert_eq!(items.as_array().unwrap().len(), 3);
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item.get("initial_password").is_none())
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn ajout_refuse_usurpation_adresses_invalides_et_doublons() {
    let (app, database) = setup().await;
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    for extra in ["commune_id", "role", "password"] {
        let mut forged = json!({"name":"Alex Dupont","email":"alex.dupont@example.test"});
        forged[extra] = json!("maire");
        assert_eq!(
            request(&app, "POST", "/members", Some(&mayor), Some(forged))
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    for email in [
        "invalide",
        "a@@exemple.fr",
        "a@.fr",
        "a@-exemple.fr",
        "a b@exemple.fr",
        "a..b@exemple.fr",
    ] {
        assert_eq!(
            request(
                &app,
                "POST",
                "/members",
                Some(&mayor),
                Some(json!({"name":"Alex Dupont","email":email}))
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &app,
            "POST",
            "/members",
            Some(&mayor),
            Some(json!({"name":"x","email":"alex@example.test"}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/members",
            Some(&mayor),
            Some(json!({"name":"Un autre nom","email":"MAIRE.BORDEAUX@DEMO.QOMUNE.FR"}))
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    // Un doublon ne remplace ni le mot de passe ni la commune du compte existant.
    let token = sign_in(&app, "maire.bordeaux@demo.qomune.fr").await;
    let (_, result, _) = request(&app, "GET", "/auth/me", Some(&token), None).await;
    assert_eq!(result["member"]["role"], "maire");
    assert_eq!(result["member"]["commune_id"], "33063");
    assert_eq!(
        database
            .collection::<Document>("members")
            .count_documents(doc! {})
            .await
            .unwrap(),
        7
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn deux_ajouts_simultanes_ne_creent_qu_un_compte() {
    let (app, database) = setup().await;
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let payload = json!({"name":"Alex Dupont","email":"alex.dupont@example.test"});
    let (first, second) = tokio::join!(
        request(
            &app,
            "POST",
            "/members",
            Some(&mayor),
            Some(payload.clone())
        ),
        request(&app, "POST", "/members", Some(&mayor), Some(payload))
    );
    let mut statuses = [first.0.as_u16(), second.0.as_u16()];
    statuses.sort();
    assert_eq!(statuses, [201, 409]);
    assert_eq!(
        database
            .collection::<Document>("members")
            .count_documents(doc! {"email":"alex.dupont@example.test"})
            .await
            .unwrap(),
        1
    );
    database.drop().await.unwrap();
}

/// Chaque élu choisit sa visibilité ; sa signature et sa commune restent imposées par sa session.
#[tokio::test]
async fn maire_et_conseiller_publient_dans_les_deux_espaces_avec_leur_signature() {
    let (app, database) = setup().await;
    for (email, name, role) in [
        ("maire.annecy@demo.qomune.fr", "Camille Martin", "maire"),
        ("conseil.annecy@demo.qomune.fr", "Léa Robert", "conseiller"),
    ] {
        let token = sign_in(&app, email).await;
        for visibility in ["public", "private"] {
            let body = json!({"title":format!("Message {visibility} du {role}"),"body":"Une publication au nom de la commune, signée par son auteur.","category":"Vie citoyenne","visibility":visibility});
            let (status, result, _) = request(
                &app,
                "POST",
                "/publications",
                Some(&token),
                Some(body.clone()),
            )
            .await;
            assert_eq!(status, StatusCode::CREATED);
            let id = result["id"].as_str().unwrap();
            let path = format!("/publications/{id}");
            let (status, publication, _) = request(&app, "GET", &path, Some(&token), None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(publication["commune_id"], "74010");
            assert_eq!(publication["author_name"], name);
            assert_eq!(publication["author_role"], role);
            assert_eq!(publication["visibility"], visibility);
            assert_eq!(
                request(&app, "GET", &path, None, None).await.0,
                if visibility == "public" {
                    StatusCode::OK
                } else {
                    StatusCode::NOT_FOUND
                }
            );
            let (_, public_feed, _) = request(&app, "GET", "/publications", None, None).await;
            assert_eq!(
                public_feed
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|publication| publication["id"] == id),
                visibility == "public"
            );
            for field in ["commune_id", "author_name", "author_role", "author_id"] {
                let mut forged = body.clone();
                forged[field] = json!("Une autre identité");
                assert_eq!(
                    request(&app, "POST", "/publications", Some(&token), Some(forged))
                        .await
                        .0,
                    StatusCode::UNPROCESSABLE_ENTITY
                );
            }
        }
    }
    database.drop().await.unwrap();
}

#[tokio::test]
async fn recherche_de_communes_bornee_par_nom_et_code() {
    let (app, database) = setup().await;
    let (status, items, _) = request(&app, "GET", "/communes?q=ANNECY&limit=1", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(items.as_array().unwrap().len(), 1);
    assert_eq!(items[0]["id"], "74010");
    let (_, items, _) = request(&app, "GET", "/communes?q=33", None, None).await;
    assert_eq!(items[0]["name"], "Bordeaux");
    let (_, items, _) = request(&app, "GET", "/communes?ids=74010,33063", None, None).await;
    assert_eq!(items.as_array().unwrap().len(), 2);
    assert_eq!(
        request(&app, "GET", "/communes?limit=1000", None, None)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(&app, "GET", "/communes?ids=code-invalide", None, None)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (_, items, _) = request(&app, "GET", "/communes?q=.%2A", None, None).await;
    // Une recherche sans caractère alphanumérique ne devient jamais une regex libre.
    assert!(items.as_array().unwrap().is_empty());
    database.drop().await.unwrap();
}

#[tokio::test]
async fn import_du_referentiel_repris_sans_modifier_les_comptes_et_publications() {
    let (app, database) = setup().await;
    let record = reference::ReferenceCommune {
        commune: Commune {
            id: "42218".into(),
            name: "Saint-Étienne".into(),
            department: "Loire · 42".into(),
            region: "Auvergne-Rhône-Alpes".into(),
        },
        search_name: reference::normalize_name("Saint-Étienne"),
    };
    let manifest = reference::Manifest {
        vintage: 2026,
        count: 1,
        sha256: "empreinte-de-test".into(),
    };
    reference::import_records(&database, vec![record.clone()], &manifest)
        .await
        .unwrap();
    reference::import_records(&database, vec![record.clone()], &manifest)
        .await
        .unwrap();
    assert_eq!(
        database
            .collection::<Document>("communes")
            .count_documents(doc! {})
            .await
            .unwrap(),
        7
    );
    assert_eq!(
        database
            .collection::<Document>("members")
            .count_documents(doc! {})
            .await
            .unwrap(),
        7
    );
    assert_eq!(
        database
            .collection::<Document>("publications")
            .count_documents(doc! {})
            .await
            .unwrap(),
        9
    );
    let (_, items, _) = request(&app, "GET", "/communes?q=saint%20etienne", None, None).await;
    assert_eq!(items[0]["id"], "42218");
    let (_, metadata, _) = request(&app, "GET", "/communes/meta", None, None).await;
    assert_eq!(metadata["total"], 7);
    assert_eq!(metadata["vintage"], 2026);
    // Un document absent est réimporté, même si la précédente opération avait son marqueur de succès.
    database
        .collection::<Document>("communes")
        .delete_one(doc! {"id":"42218"})
        .await
        .unwrap();
    reference::import_records(&database, vec![record], &manifest)
        .await
        .unwrap();
    assert!(
        database
            .collection::<Document>("communes")
            .find_one(doc! {"id":"42218"})
            .await
            .unwrap()
            .is_some()
    );
    sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    database.drop().await.unwrap();
}

async fn add_registration_commune(database: &Database) {
    database.collection::<Document>("communes").insert_one(doc!{"id":"42218","name":"Saint-Étienne","department":"Loire · 42","region":"Auvergne-Rhône-Alpes","search_name":"saint etienne"}).await.unwrap();
}

#[tokio::test]
async fn maire_inscrit_sans_verification_dans_le_prototype() {
    let (app, database) = setup().await;
    add_registration_commune(&database).await;
    let input = json!({"name":"Alex Dupont","email":"nouveau.maire@example.test","password":"Mot-de-passe-de-test-2026","commune_id":"42218"});
    let (status, _, cookie) = request(&app, "POST", "/auth/register", None, Some(input)).await;
    assert_eq!(status, StatusCode::CREATED);
    let cookie = cookie.unwrap();
    let (_, profile, _) = request(&app, "GET", "/auth/me", Some(&cookie), None).await;
    assert_eq!(profile["member"]["role"], "maire");
    assert_eq!(profile["member"]["commune_id"], "42218");
    let stored = database
        .collection::<Document>("members")
        .find_one(doc! {"email":"nouveau.maire@example.test"})
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.get_str("verification_status").unwrap(),
        "unverified_prototype"
    );
    assert_ne!(
        stored.get_str("password_hash").unwrap(),
        "Mot-de-passe-de-test-2026"
    );
    let new_member = json!({"name":"Un conseiller","email":"conseiller.nouveau@example.test"});
    assert_eq!(
        request(&app, "POST", "/members", Some(&cookie), Some(new_member))
            .await
            .0,
        StatusCode::CREATED
    );
    assert_eq!(
        request(&app, "GET", "/publications/demo-8", Some(&cookie), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn inscription_de_maire_validee_et_refusee_hors_prototype() {
    let (app, database) = setup().await;
    add_registration_commune(&database).await;
    let input = json!({"name":"Alex Dupont","email":"nouveau.maire@example.test","password":"Mot-de-passe-de-test-2026","commune_id":"42218"});
    let registration = serde_json::from_value::<registration::Registration>(input.clone()).unwrap();
    let result = registration::register_for_mode(database.clone(), registration, false).await;
    assert_eq!(result.unwrap_err().0, StatusCode::FORBIDDEN);
    let mut invalid = input.clone();
    invalid["password"] = json!("court");
    assert_eq!(
        request(&app, "POST", "/auth/register", None, Some(invalid))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let mut invalid = input.clone();
    invalid["commune_id"] = json!("00000");
    assert_eq!(
        request(&app, "POST", "/auth/register", None, Some(invalid))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let mut forged = input.clone();
    forged["role"] = json!("administrateur");
    assert_eq!(
        request(&app, "POST", "/auth/register", None, Some(forged))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut duplicate = input;
    duplicate["commune_id"] = json!("74010");
    assert_eq!(
        request(&app, "POST", "/auth/register", None, Some(duplicate))
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        database
            .collection::<Document>("members")
            .count_documents(doc! {})
            .await
            .unwrap(),
        7
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn une_seule_inscription_de_maire_par_commune_en_cas_de_concurrence() {
    let (app, database) = setup().await;
    add_registration_commune(&database).await;
    let first = json!({"name":"Alex Dupont","email":"premier.maire@example.test","password":"Mot-de-passe-de-test-2026","commune_id":"42218"});
    let mut second = first.clone();
    second["email"] = json!("autre.maire@example.test");
    let (first, second) = tokio::join!(
        request(&app, "POST", "/auth/register", None, Some(first)),
        request(&app, "POST", "/auth/register", None, Some(second))
    );
    let mut statuses = [first.0.as_u16(), second.0.as_u16()];
    statuses.sort();
    assert_eq!(statuses, [201, 409]);
    assert_eq!(
        database
            .collection::<Document>("members")
            .count_documents(doc! {"role":"maire","commune_id":"42218"})
            .await
            .unwrap(),
        1
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn echanges_inaccessibles_aux_visiteurs_et_aux_autres_conseils() {
    let (app, database) = setup().await;
    let other = sign_in(&app, "maire.bordeaux@demo.qomune.fr").await;
    let owner = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    for publication in ["demo-1", "demo-8"] {
        for (method, suffix, payload) in [
            ("GET", "discussion", None),
            (
                "POST",
                "comments",
                Some(json!({"body":"Un message interne"})),
            ),
            ("POST", "reaction", Some(json!({"reaction":"like"}))),
        ] {
            let path = format!("/publications/{publication}/{suffix}");
            assert_eq!(
                request(&app, method, &path, None, payload.clone()).await.0,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                request(&app, method, &path, Some(&other), payload).await.0,
                StatusCode::NOT_FOUND
            );
        }
        let path = format!("/publications/{publication}/discussion");
        assert_eq!(
            request(&app, "GET", &path, Some(&owner), None).await.0,
            StatusCode::OK
        );
    }
    assert_eq!(
        request(
            &app,
            "GET",
            "/publications/inexistante/discussion",
            Some(&owner),
            None
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn commentaires_signes_et_conserves_sans_exposition_dans_le_fil_public() {
    let (app, database) = setup().await;
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let adviser = sign_in(&app, "conseil.annecy@demo.qomune.fr").await;
    for (token, name, role) in [
        (&mayor, "Camille Martin", "maire"),
        (&adviser, "Léa Robert", "conseiller"),
    ] {
        let (status, comment, _) = request(
            &app,
            "POST",
            "/publications/demo-1/comments",
            Some(token),
            Some(json!({"body":"  Un échange réservé au conseil.  "})),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(comment["author_name"], name);
        assert_eq!(comment["author_role"], role);
        assert_eq!(comment["commune_id"], "74010");
        assert_eq!(comment["body"], "Un échange réservé au conseil.");
    }
    for body in [" ".to_owned(), "x".repeat(2001)] {
        assert_eq!(
            request(
                &app,
                "POST",
                "/publications/demo-1/comments",
                Some(&mayor),
                Some(json!({"body":body}))
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    for field in ["author_id", "author_name", "commune_id"] {
        let mut forged = json!({"body":"Un commentaire"});
        forged[field] = json!("autre-identité");
        assert_eq!(
            request(
                &app,
                "POST",
                "/publications/demo-1/comments",
                Some(&adviser),
                Some(forged)
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let restarted = super::app(database.clone());
    let (_, discussion, _) = request(
        &restarted,
        "GET",
        "/publications/demo-1/discussion",
        Some(&adviser),
        None,
    )
    .await;
    assert_eq!(discussion["comments_count"], 2);
    assert_eq!(discussion["comments"].as_array().unwrap().len(), 2);
    let (_, publication, _) = request(&app, "GET", "/publications/demo-1", None, None).await;
    assert!(
        publication.get("comments").is_none()
            && publication.get("reactions").is_none()
            && publication.get("comments_count").is_none()
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn reaction_unique_modifiable_et_retirable_par_son_auteur() {
    let (app, database) = setup().await;
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let adviser = sign_in(&app, "conseil.annecy@demo.qomune.fr").await;
    let path = "/publications/demo-8/reaction";
    for _ in 0..2 {
        let (status, summary, _) = request(
            &app,
            "POST",
            path,
            Some(&mayor),
            Some(json!({"reaction":"like"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(summary["total"], 1);
        assert_eq!(summary["counts"]["like"], 1);
    }
    let (_, summary, _) = request(
        &app,
        "POST",
        path,
        Some(&mayor),
        Some(json!({"reaction":"love"})),
    )
    .await;
    assert_eq!(summary["total"], 1);
    assert_eq!(summary["mine"], "love");
    assert_eq!(summary["counts"]["like"], 0);
    let (_, summary, _) = request(
        &app,
        "POST",
        path,
        Some(&adviser),
        Some(json!({"reaction":"haha"})),
    )
    .await;
    assert_eq!(summary["total"], 2);
    assert_eq!(summary["mine"], "haha");
    let (_, summary, _) = request(
        &app,
        "POST",
        path,
        Some(&mayor),
        Some(json!({"reaction":null})),
    )
    .await;
    assert_eq!(summary["total"], 1);
    assert_eq!(summary["counts"]["love"], 0);
    assert!(summary["mine"].is_null());
    assert_eq!(summary["members"][0]["author_name"], "Léa Robert");
    assert_eq!(
        request(
            &app,
            "POST",
            path,
            Some(&mayor),
            Some(json!({"reaction":"invalide"}))
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        request(
            &app,
            "POST",
            path,
            Some(&mayor),
            Some(json!({"reaction":null,"member_id":"conseiller-74010"}))
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn reactions_simultanees_ne_creent_pas_de_doublon() {
    let (app, database) = setup().await;
    let token = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let path = "/publications/demo-8/reaction";
    let (first, second) = tokio::join!(
        request(
            &app,
            "POST",
            path,
            Some(&token),
            Some(json!({"reaction":"like"}))
        ),
        request(
            &app,
            "POST",
            path,
            Some(&token),
            Some(json!({"reaction":"wow"}))
        )
    );
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(second.0, StatusCode::OK);
    assert_eq!(
        database
            .collection::<Document>("reactions")
            .count_documents(doc! {"publication_id":"demo-8","member_id":"maire-74010"})
            .await
            .unwrap(),
        1
    );
    let (_, discussion, _) = request(
        &app,
        "GET",
        "/publications/demo-8/discussion",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(discussion["reactions"]["total"], 1);
    database.drop().await.unwrap();
}

#[tokio::test]
async fn pagination_des_commentaires_sans_perte_ni_fuite() {
    let (app, database) = setup().await;
    let token = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    let comments:Vec<_>=(0..53).map(|index|doc!{"id":Uuid::new_v4().to_string(),"publication_id":"demo-8","commune_id":"74010","author_id":"maire-74010","author_name":"Camille Martin","author_role":"maire","body":format!("Commentaire {index}"),"created_at":"2026-10-29T12:00:00Z"}).collect();
    database
        .collection::<Document>("comments")
        .insert_many(comments)
        .await
        .unwrap();
    let (_, first, _) = request(
        &app,
        "GET",
        "/publications/demo-8/discussion",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(first["comments_count"], 53);
    assert_eq!(first["comments"].as_array().unwrap().len(), 50);
    let cursor = first["next_cursor"].as_str().unwrap().replace('|', "%7C");
    let (_, second, _) = request(
        &app,
        "GET",
        &format!("/publications/demo-8/discussion?before={cursor}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(second["comments"].as_array().unwrap().len(), 3);
    assert!(second["next_cursor"].is_null());
    let ids: std::collections::HashSet<_> = first["comments"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second["comments"].as_array().unwrap())
        .map(|comment| comment["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 53);
    let (_, unrelated, _) = request(
        &app,
        "GET",
        &format!("/publications/demo-1/discussion?before={cursor}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(unrelated["comments_count"], 0);
    assert!(unrelated["comments"].as_array().unwrap().is_empty());
    assert_eq!(
        request(
            &app,
            "GET",
            "/publications/demo-8/discussion?before=invalide",
            Some(&token),
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn administration_authentification_independante_et_statistiques_agregees() {
    let (app, database) = setup().await;
    assert_eq!(
        request(&app, "GET", "/admin/statistics", None, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    assert_eq!(
        request(&app, "GET", "/admin/statistics", Some(&mayor), None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/admin/login",
            None,
            Some(json!({"password":"absent"}))
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    administration::configure(&database, "mot-de-passe-admin-test-2026")
        .await
        .unwrap();
    assert_eq!(
        request(
            &app,
            "POST",
            "/admin/login",
            None,
            Some(json!({"password":"incorrect"}))
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (status, _, cookie) = request(
        &app,
        "POST",
        "/admin/login",
        None,
        Some(json!({"password":"mot-de-passe-admin-test-2026"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = cookie.unwrap();
    let (status, stats, _) =
        request(&app, "GET", "/admin/statistics?days=30", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stats["timeline"].as_array().unwrap().len(), 30);
    assert_eq!(stats["totals"]["public"], 7);
    assert_eq!(stats["totals"]["private"], 2);
    assert_eq!(stats["totals"]["members"], 7);
    assert_eq!(stats["totals"]["registered_communes"], 6);
    assert!(!stats.to_string().contains("author_name"));
    assert!(!stats.to_string().contains("password"));
    assert!(!stats.to_string().contains("@demo"));
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
    assert_eq!(
        request(
            &app,
            "GET",
            "/admin/statistics?days=365",
            Some(&token),
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    request(&app, "POST", "/admin/logout", Some(&token), Some(json!({}))).await;
    assert_eq!(
        request(&app, "GET", "/admin/statistics", Some(&token), None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn administration_periodes_vides_et_revocation_des_sessions() {
    let (app, database) = setup().await;
    administration::configure(&database, "mot-de-passe-admin-test-2026")
        .await
        .unwrap();
    let (_, _, cookie) = request(
        &app,
        "POST",
        "/admin/login",
        None,
        Some(json!({"password":"mot-de-passe-admin-test-2026"})),
    )
    .await;
    let token = cookie.unwrap();
    database
        .collection::<Document>("publications")
        .delete_many(doc! {})
        .await
        .unwrap();
    let today = Utc::now().date_naive();
    for (id, date, visibility) in [
        ("first", today - Duration::days(6), "public"),
        ("today", today, "private"),
        ("old", today - Duration::days(7), "public"),
        ("future", today + Duration::days(1), "private"),
    ] {
        database.collection::<Document>("publications").insert_one(doc! {"id":id,"commune_id":"74010","category":"Culture","published_at":format!("{date}T12:00:00+00:00"),"visibility":visibility}).await.unwrap();
    }
    let (_, stats, _) = request(&app, "GET", "/admin/statistics?days=7", Some(&token), None).await;
    let timeline = stats["timeline"].as_array().unwrap();
    assert_eq!(timeline.len(), 7);
    assert_eq!(timeline[0]["public"], 1);
    assert_eq!(timeline[6]["private"], 1);
    assert_eq!(timeline[1]["public"], 0);
    assert_eq!(stats["categories"][0]["count"], 2);
    assert_eq!(stats["top_communes"][0]["count"], 2);
    database
        .collection::<Document>("publications")
        .delete_many(doc! {})
        .await
        .unwrap();
    let (_, empty, _) = request(&app, "GET", "/admin/statistics?days=90", Some(&token), None).await;
    assert_eq!(empty["timeline"].as_array().unwrap().len(), 90);
    assert!(empty["categories"].as_array().unwrap().is_empty());
    assert_eq!(empty["totals"]["public"], 0);
    administration::configure(&database, "un-autre-secret-admin-2026")
        .await
        .unwrap();
    assert_eq!(
        request(&app, "GET", "/admin/statistics", Some(&token), None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    database.drop().await.unwrap();
}

async fn record_visit(app: &Router, key: Option<&str>, body: Value) -> StatusCode {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/internal/visits")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(key) = key {
        builder = builder.header("x-qomune-metrics-key", key);
    }
    app.clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
        .status()
}
#[tokio::test]
async fn visites_anonymes_protegees_et_sans_donnees_individuelles() {
    let (app, database) = setup().await;
    let key = "cle-de-mesure-de-test-avec-32-caracteres";
    visits::configure(&database, key).await.unwrap();
    assert_eq!(
        record_visit(&app, None, json!({"page":"public"})).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        record_visit(
            &app,
            Some("mauvaise-cle-longue-de-plus-de-32-caracteres"),
            json!({"page":"public"})
        )
        .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        record_visit(&app, Some(key), json!({"page":"inconnue"})).await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        record_visit(&app, Some(key), json!({"page":"public","ip":"127.0.0.1"})).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for page in ["public", "admin_demo", "admin_private"] {
        assert_eq!(
            record_visit(&app, Some(key), json!({"page":page})).await,
            StatusCode::NO_CONTENT
        );
    }
    assert_eq!(
        request(&app, "GET", "/admin/statistics", None, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    assert_eq!(
        request(&app, "GET", "/admin/statistics", Some(&mayor), None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    administration::configure(&database, "secret-administrateur-test-2026")
        .await
        .unwrap();
    let (_, _, cookie) = request(
        &app,
        "POST",
        "/admin/login",
        None,
        Some(json!({"password":"secret-administrateur-test-2026"})),
    )
    .await;
    let (status, stats, _) = request(
        &app,
        "GET",
        "/admin/statistics?days=7",
        cookie.as_deref(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stats["visits"]["total"], 3);
    let days = stats["visits"]["timeline"].as_array().unwrap();
    assert_eq!(days.len(), 7);
    assert_eq!(days[0]["measured"], false);
    assert_eq!(days[6]["measured"], true);
    assert_eq!(days[6]["public"], 1);
    assert_eq!(days[6]["admin_demo"], 1);
    assert_eq!(days[6]["admin_private"], 1);
    let row = database
        .collection::<Document>("daily_visits")
        .find_one(doc! {})
        .await
        .unwrap()
        .unwrap();
    assert!(row.keys().all(|key| {
        [
            "_id",
            "date",
            "expires_at",
            "public",
            "admin_demo",
            "admin_private",
        ]
        .contains(&key.as_str())
    }));
    assert!(!stats.to_string().contains(key));
    database.drop().await.unwrap();
}
#[tokio::test]
async fn visites_simultanees_et_periodes_bornees() {
    let (app, database) = setup().await;
    let key = "cle-de-mesure-de-test-avec-32-caracteres";
    visits::configure(&database, key).await.unwrap();
    let results = futures_util::future::join_all(
        (0..20).map(|_| record_visit(&app, Some(key), json!({"page":"public"}))),
    )
    .await;
    assert!(results.iter().all(|s| *s == StatusCode::NO_CONTENT));
    let today = Utc::now().date_naive();
    let start = today - Duration::days(6);
    for (date, n) in [
        (start, 4_i64),
        (start - Duration::days(1), 8),
        (today + Duration::days(1), 16),
    ] {
        database
            .collection::<Document>("daily_visits")
            .insert_one(doc! {"_id":date.to_string(),"public":n})
            .await
            .unwrap();
    }
    let summary = visits::summary(&database, start, today).await.unwrap();
    assert_eq!(summary["total"], 24);
    assert_eq!(summary["timeline"][6]["public"], 20);
    assert_eq!(summary["timeline"][0]["public"], 4);
    database.drop().await.unwrap();
}

#[tokio::test]
async fn diffusion_nationale_unique_visible_dans_toutes_les_communes_et_signee() {
    let (app, database) = setup().await;
    government::initialize(&database).await.unwrap();
    let (_, _, cookie) = request(
        &app,
        "POST",
        "/government-demo/login",
        None,
        Some(json!({
            "email":"sebastien-lecornu@gouvernement.demo.qomune.fr", "password":"gouvernement-demo"
        })),
    )
    .await;
    let cookie = cookie.unwrap();
    let input = json!({"title":"Un essai national", "body":"Message fictif destiné à toutes les communes.", "category":"Services publics", "request_id":Uuid::new_v4().to_string()});
    assert_eq!(
        request(
            &app,
            "POST",
            "/government-demo/publications",
            None,
            Some(input.clone())
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let mayor = sign_in(&app, "maire.annecy@demo.qomune.fr").await;
    assert_eq!(
        request(
            &app,
            "POST",
            "/government-demo/publications",
            Some(&mayor),
            Some(input.clone())
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (status, result, _) = request(
        &app,
        "POST",
        "/government-demo/publications",
        Some(&cookie),
        Some(input.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = result["id"].as_str().unwrap();
    // Un double clic ou un nouvel essai réseau ne crée pas de copies supplémentaires.
    let (a, b) = tokio::join!(
        request(
            &app,
            "POST",
            "/government-demo/publications",
            Some(&cookie),
            Some(input.clone())
        ),
        request(
            &app,
            "POST",
            "/government-demo/publications",
            Some(&cookie),
            Some(input.clone())
        )
    );
    assert_eq!(a.0, StatusCode::CREATED);
    assert_eq!(b.1["id"], id);
    assert_eq!(
        database
            .collection::<Document>("publications")
            .count_documents(doc! {"scope":"national"})
            .await
            .unwrap(),
        1
    );
    for commune in ["74010", "33063", "2A004", "97105"] {
        let (status, posts, _) = request(
            &app,
            "GET",
            &format!("/publications?commune={commune}&q=essai&category=Services%20publics"),
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let posts = posts.as_array().unwrap();
        assert_eq!(
            posts.len(),
            1,
            "La recherche et la catégorie ne doivent pas écraser le filtre géographique"
        );
        assert_eq!(posts[0]["id"], id);
        assert_eq!(posts[0]["scope"], "national");
        assert_eq!(posts[0]["is_demo"], true);
        assert_eq!(posts[0]["author_name"], "Sébastien Lecornu (compte fictif)");
        assert_eq!(posts[0]["author_role"], "Premier ministre");
        assert_eq!(
            posts[0]["author_profile"],
            "/gouvernement/sebastien-lecornu"
        );
        assert!(
            posts[0]["body"]
                .as_str()
                .unwrap()
                .starts_with("Publication fictive de démonstration.")
        );
        assert!(
            posts[0]["title"]
                .as_str()
                .unwrap()
                .starts_with("Démonstration — ")
        );
    }
    let all = request(&app, "GET", "/publications", None, None).await.1;
    assert_eq!(
        all.as_array()
            .unwrap()
            .iter()
            .filter(|p| p["id"] == id)
            .count(),
        1
    );
    let council = request(
        &app,
        "GET",
        "/publications?scope=council",
        Some(&mayor),
        None,
    )
    .await
    .1;
    assert!(!council.as_array().unwrap().iter().any(|p| p["id"] == id));
    assert_eq!(
        request(&app, "GET", &format!("/publications/{id}"), None, None)
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("/publications/{id}/discussion"),
            Some(&cookie),
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let mut forged = input.clone();
    forged["author_name"] = json!("Autre personne");
    assert_eq!(
        request(
            &app,
            "POST",
            "/government-demo/publications",
            Some(&cookie),
            Some(forged)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut invalid_input = input.clone();
    invalid_input["title"] = json!("x");
    assert_eq!(
        request(
            &app,
            "POST",
            "/government-demo/publications",
            Some(&cookie),
            Some(invalid_input)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (_, _, other) = request(&app,"POST","/government-demo/login",None,Some(json!({"email":"laurent-nunez@gouvernement.demo.qomune.fr","password":"gouvernement-demo"}))).await;
    let other = other.unwrap();
    assert!(
        request(
            &app,
            "GET",
            "/government-demo/publications",
            Some(&other),
            None
        )
        .await
        .1
        .as_array()
        .unwrap()
        .is_empty()
    );
    let second = request(
        &app,
        "POST",
        "/government-demo/publications",
        Some(&other),
        Some(input.clone()),
    )
    .await
    .1;
    assert_ne!(
        second["id"], id,
        "Deux auteurs ne partagent jamais leur identifiant de diffusion"
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/government-demo/publications",
            Some(&cookie),
            None
        )
        .await
        .1
        .as_array()
        .unwrap()
        .len(),
        1
    );
    database
        .collection::<Document>("government_demo_sessions")
        .update_many(
            doc! {},
            doc! {"$set":{"expires_at":DateTime::from_millis(0)}},
        )
        .await
        .unwrap();
    assert_eq!(
        request(
            &app,
            "POST",
            "/government-demo/publications",
            Some(&cookie),
            Some(input)
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    database.drop().await.unwrap();
}

#[tokio::test]
async fn gouvernement_peut_cibler_une_commune_sans_diffuser_aux_autres() {
    let (app, database) = setup().await;
    government::initialize(&database).await.unwrap();
    let (_,_,cookie) = request(&app,"POST","/government-demo/login",None,Some(json!({"email":"sebastien-lecornu@gouvernement.demo.qomune.fr","password":"gouvernement-demo"}))).await;
    let cookie = cookie.unwrap();
    let payload = json!({"title":"Un message pour Annecy","body":"Une publication fictive destinée uniquement à Annecy.","category":"Services publics","request_id":Uuid::new_v4().to_string(),"commune_id":"74010"});
    let (status, receipt, _) = request(
        &app,
        "POST",
        "/government-demo/publications",
        Some(&cookie),
        Some(payload.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(receipt["scope"], "commune");
    assert_eq!(receipt["commune_name"], "Annecy");
    let id = receipt["id"].as_str().unwrap();
    for (commune, expected) in [("74010", true), ("33063", false), ("44109", false)] {
        let (_, posts, _) = request(
            &app,
            "GET",
            &format!("/publications?commune={commune}"),
            None,
            None,
        )
        .await;
        assert_eq!(
            posts
                .as_array()
                .unwrap()
                .iter()
                .any(|post| post["id"] == id),
            expected
        );
    }
    let post = request(&app, "GET", &format!("/publications/{id}"), None, None)
        .await
        .1;
    assert_eq!(post["commune_id"], "74010");
    assert_eq!(post["visibility"], "public");
    assert_eq!(post["is_demo"], true);
    assert_eq!(post["author_role"], "Premier ministre");
    let history = request(
        &app,
        "GET",
        "/government-demo/publications",
        Some(&cookie),
        None,
    )
    .await
    .1;
    assert_eq!(history.as_array().unwrap().len(), 1);
    assert_eq!(history[0]["commune_name"], "Annecy");
    // Une nouvelle tentative ne change ni la destination initiale ni le reçu.
    let mut retry = payload.clone();
    retry["commune_id"] = json!("33063");
    let replay = request(
        &app,
        "POST",
        "/government-demo/publications",
        Some(&cookie),
        Some(retry),
    )
    .await
    .1;
    assert_eq!(replay["commune_id"], "74010");
    assert_eq!(
        database
            .collection::<Document>("publications")
            .count_documents(doc! {"id":id})
            .await
            .unwrap(),
        1
    );
    for code in ["99999", "", "74010,33063"] {
        let mut invalid = payload.clone();
        invalid["commune_id"] = json!(code);
        invalid["request_id"] = json!(Uuid::new_v4().to_string());
        assert_eq!(
            request(
                &app,
                "POST",
                "/government-demo/publications",
                Some(&cookie),
                Some(invalid)
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    database.drop().await.unwrap();
}
