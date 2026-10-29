//! Échanges internes autour d'une publication, y compris lorsqu'elle est publique.
//! Aucun commentaire, nom de réacteur ou compteur n'est accessible au visiteur public.
use super::*;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ReactionKind {
    Like,
    Love,
    Haha,
    Wow,
    Sad,
    Angry,
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
    id: String,
    publication_id: String,
    commune_id: String,
    author_id: String,
    author_name: String,
    author_role: String,
    body: String,
    created_at: String,
}
#[derive(Serialize, Deserialize)]
struct Reaction {
    publication_id: String,
    commune_id: String,
    member_id: String,
    author_name: String,
    author_role: String,
    reaction: ReactionKind,
}
#[derive(Serialize)]
pub struct ReactionSummary {
    counts: std::collections::BTreeMap<ReactionKind, u64>,
    total: usize,
    mine: Option<ReactionKind>,
    members: Vec<Reaction>,
}

/// Chaque route vérifie la session ET la commune de la publication avant de lire ou écrire les échanges.
async fn authorize(database: &Database, headers: &HeaderMap, id: &str) -> ApiResult<Member> {
    let user = member(database, headers).await?.ok_or_else(unauthorized)?;
    let publication = database
        .collection::<Document>("publications")
        .find_one(
            doc! {"id":id,"commune_id":&user.commune_id,"visibility":{"$in":["public","private"]}},
        )
        .await?;
    if publication.is_none() {
        return Err(not_found());
    }
    Ok(user)
}

pub async fn initialize(database: &Database) -> mongodb::error::Result<()> {
    database
        .collection::<Document>("comments")
        .create_index(
            IndexModel::builder()
                .keys(doc! {"publication_id":1,"commune_id":1,"created_at":-1,"id":-1})
                .build(),
        )
        .await?;
    database
        .collection::<Document>("reactions")
        .create_index(
            IndexModel::builder()
                .keys(doc! {"publication_id":1,"member_id":1})
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    Ok(())
}

async fn reaction_summary(
    database: &Database,
    id: &str,
    user: &Member,
) -> ApiResult<ReactionSummary> {
    let members: Vec<Reaction> = database
        .collection("reactions")
        .find(doc! {"publication_id":id,"commune_id":&user.commune_id})
        .sort(doc! {"author_name":1,"member_id":1})
        .await?
        .try_collect()
        .await?;
    let mut counts = std::collections::BTreeMap::from([
        (ReactionKind::Like, 0),
        (ReactionKind::Love, 0),
        (ReactionKind::Haha, 0),
        (ReactionKind::Wow, 0),
        (ReactionKind::Sad, 0),
        (ReactionKind::Angry, 0),
    ]);
    for reaction in &members {
        *counts.entry(reaction.reaction).or_default() += 1;
    }
    let mine = members
        .iter()
        .find(|r| r.member_id == user.id)
        .map(|r| r.reaction);
    Ok(ReactionSummary {
        total: members.len(),
        counts,
        mine,
        members,
    })
}

#[derive(Deserialize)]
pub struct CommentQuery {
    before: Option<String>,
}
#[derive(Serialize)]
pub struct Discussion {
    comments: Vec<Comment>,
    comments_count: u64,
    next_cursor: Option<String>,
    reactions: ReactionSummary,
}

/// Affiche les 50 commentaires les plus récents, dans l'ordre chronologique, avec un curseur vers les précédents.
pub async fn read(
    State(database): State<Database>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<CommentQuery>,
) -> ApiResult<Json<Discussion>> {
    let user = authorize(&database, &headers, &id).await?;
    let base = doc! {"publication_id":&id,"commune_id":&user.commune_id};
    let comments_count = database
        .collection::<Document>("comments")
        .count_documents(base.clone())
        .await?;
    let mut filter = base;
    if let Some(cursor) = query.before {
        if cursor.len() > 150 {
            return Err(invalid("Curseur de commentaires invalide."));
        }
        let Some((date, comment_id)) = cursor.split_once('|') else {
            return Err(invalid("Curseur de commentaires invalide."));
        };
        if chrono::DateTime::parse_from_rfc3339(date).is_err()
            || Uuid::parse_str(comment_id).is_err()
        {
            return Err(invalid("Curseur de commentaires invalide."));
        }
        filter.insert(
            "$or",
            vec![
                doc! {"created_at":{"$lt":date}},
                doc! {"created_at":date,"id":{"$lt":comment_id}},
            ],
        );
    }
    let mut comments: Vec<Comment> = database
        .collection("comments")
        .find(filter)
        .sort(doc! {"created_at":-1,"id":-1})
        .limit(51)
        .await?
        .try_collect()
        .await?;
    let more = comments.len() > 50;
    comments.truncate(50);
    let next_cursor = if more {
        comments
            .last()
            .map(|c| format!("{}|{}", c.created_at, c.id))
    } else {
        None
    };
    comments.reverse();
    let reactions = reaction_summary(&database, &id, &user).await?;
    Ok(Json(Discussion {
        comments,
        comments_count,
        next_cursor,
        reactions,
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewComment {
    body: String,
}

pub async fn comment(
    State(database): State<Database>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<NewComment>,
) -> ApiResult<(StatusCode, Json<Comment>)> {
    let user = authorize(&database, &headers, &id).await?;
    let body = input.body.trim();
    if !(1..=2000).contains(&body.chars().count()) {
        return Err(invalid(
            "Le commentaire doit contenir de 1 à 2 000 caractères.",
        ));
    }
    let comment = Comment {
        id: Uuid::new_v4().to_string(),
        publication_id: id,
        commune_id: user.commune_id,
        author_id: user.id,
        author_name: user.name,
        author_role: user.role,
        body: body.into(),
        created_at: Utc::now().to_rfc3339(),
    };
    database
        .collection::<Comment>("comments")
        .insert_one(&comment)
        .await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetReaction {
    reaction: Option<ReactionKind>,
}

/// Une réaction par membre et publication. La valeur nulle supprime uniquement sa propre réaction.
pub async fn react(
    State(database): State<Database>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<SetReaction>,
) -> ApiResult<Json<ReactionSummary>> {
    let user = authorize(&database, &headers, &id).await?;
    let key = format!("{id}:{}", user.id);
    if let Some(kind) = input.reaction {
        // Le document est identifié de façon déterministe pour que les mises à jour restent idempotentes.
        let kind = serde_json::to_value(kind)
            .expect("Réaction sérialisable")
            .as_str()
            .expect("Réaction textuelle")
            .to_owned();
        let update = doc! {"$set":{"publication_id":&id,"commune_id":&user.commune_id,"member_id":&user.id,"author_name":&user.name,"author_role":&user.role,"reaction":kind}};
        let collection = database.collection::<Document>("reactions");
        if let Err(error) = collection
            .update_one(doc! {"_id":&key}, update.clone())
            .upsert(true)
            .await
        {
            // Deux premières réactions simultanées du même membre ne doivent pas créer deux documents.
            if matches!(error.kind.as_ref(),mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(write)) if write.code==11000)
            {
                collection.update_one(doc! {"_id":&key}, update).await?;
            } else {
                return Err(error.into());
            }
        }
    } else {
        database
            .collection::<Document>("reactions")
            .delete_one(doc! {"_id":key,"member_id":&user.id})
            .await?;
    }
    Ok(Json(reaction_summary(&database, &id, &user).await?))
}
