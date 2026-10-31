//! Prix déclarés des carburants : minima par type, dans les contours exacts de la commune.
//! Les données publiques sont mutualisées en cache, sans localisation du visiteur.
use super::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock, Weak},
};
use tokio::sync::Mutex;

pub const SOURCE_URL: &str =
    "https://www.data.gouv.fr/datasets/prix-des-carburants-en-france-flux-instantane-v2-amelioree";
const EXPORT_URL: &str = "https://data.economie.gouv.fr/api/explore/v2.1/catalog/datasets/prix-des-carburants-en-france-flux-instantane-v2/exports/json";
const FIELDS: &str = "id,adresse,ville,geom,carburants_disponibles,gazole_prix,gazole_maj,sp95_prix,sp95_maj,sp98_prix,sp98_maj,e10_prix,e10_maj,e85_prix,e85_maj,gplc_prix,gplc_maj";
const NATIONAL_KEY: &str = "fuel-national-v1";
const FUELS: [(&str, &str); 6] = [
    ("gazole", "Gazole"),
    ("sp95", "SP95"),
    ("sp98", "SP98"),
    ("e10", "E10"),
    ("e85", "E85"),
    ("gplc", "GPLc"),
];
type FetchLock = Mutex<()>;
static LOCKS: OnceLock<Mutex<HashMap<String, Weak<FetchLock>>>> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct CachedData {
    #[serde(rename = "_id")]
    pub key: String,
    pub data: Value,
    pub fetched_at: String,
    pub expires_at: DateTime,
}
#[derive(Serialize, Clone)]
pub struct StationPrice {
    id: String,
    address: String,
    city: String,
    updated_at: String,
    older_than_week: bool,
    distance_km: Option<f64>,
}
#[derive(Serialize)]
pub struct FuelMinimum {
    fuel: String,
    search_scope: String,
    price_per_litre: Option<f64>,
    stations: Vec<StationPrice>,
}
#[derive(Serialize)]
pub struct CommuneFuel {
    commune_id: String,
    commune_name: String,
    station_count: usize,
    nearby_radius_km: f64,
    prices: Vec<FuelMinimum>,
    fetched_at: String,
    source_url: &'static str,
}
fn unavailable() -> ApiError {
    ApiError(StatusCode::SERVICE_UNAVAILABLE, "Les prix des carburants sont momentanément indisponibles. Réessayez dans quelques instants.".into())
}

pub async fn initialize(database: &Database) -> mongodb::error::Result<()> {
    database
        .collection::<Document>("fuel_cache")
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

/// Les appels simultanés partagent une actualisation ; les verrous inutilisés ne s'accumulent pas.
async fn fetch_lock(key: &str) -> Arc<FetchLock> {
    let mut locks = LOCKS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .await;
    locks.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = locks.get(key).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(Mutex::new(()));
    locks.insert(key.to_owned(), Arc::downgrade(&lock));
    lock
}
async fn cached(database: &Database, key: &str, code: Option<&str>) -> ApiResult<CachedData> {
    let cache = database.collection::<CachedData>("fuel_cache");
    if let Some(value) = cache
        .find_one(doc! {"_id":key,"expires_at":{"$gt":DateTime::now()}})
        .await?
    {
        return Ok(value);
    }
    let lock = fetch_lock(key).await;
    let _guard = lock.lock().await;
    if let Some(value) = cache
        .find_one(doc! {"_id":key,"expires_at":{"$gt":DateTime::now()}})
        .await?
    {
        return Ok(value);
    }
    let client = reqwest::Client::builder()
        .https_only(true)
        .gzip(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Qomune/0.1 (donnees publiques carburants)")
        .build()
        .map_err(|_| unavailable())?;
    // Les deux hôtes sont fixes ; aucun URL, cookie ou secret fourni par le visiteur n'est transmis.
    let request = match code {
        Some(code) => client
            .get(format!("https://geo.api.gouv.fr/communes/{code}"))
            .query(&[("format", "geojson"), ("geometry", "contour")]),
        None => client.get(EXPORT_URL).query(&[("select", FIELDS)]),
    };
    let mut response = request
        .send()
        .await
        .map_err(|_| unavailable())?
        .error_for_status()
        .map_err(|_| unavailable())?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| unavailable())? {
        if bytes.len() + chunk.len() > 12_000_000 {
            return Err(unavailable());
        }
        bytes.extend_from_slice(&chunk);
    }
    let data: Value = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
    if let Some(code) = code {
        if data["properties"]["code"] != code || Boundary::parse(&data["geometry"]).is_none() {
            return Err(unavailable());
        }
    } else if data
        .as_array()
        .is_none_or(|items| items.is_empty() || items.len() > 25000)
    {
        return Err(unavailable());
    }
    let ttl = if code.is_some() { 86400 } else { 600 };
    let value = CachedData {
        key: key.into(),
        data,
        fetched_at: Utc::now().to_rfc3339(),
        expires_at: DateTime::from_millis((Utc::now() + Duration::seconds(ttl)).timestamp_millis()),
    };
    cache
        .replace_one(doc! {"_id":key}, &value)
        .upsert(true)
        .await?;
    Ok(value)
}

/// Un code postal ne définit pas une commune : les stations sont rattachées par leurs coordonnées.
struct Boundary {
    polygons: Vec<Vec<Vec<[f64; 2]>>>,
    bbox: [f64; 4],
}
impl Boundary {
    fn parse(geometry: &Value) -> Option<Self> {
        let polygons: Vec<Vec<Vec<[f64; 2]>>> = match geometry["type"].as_str()? {
            "Polygon" => vec![serde_json::from_value(geometry["coordinates"].clone()).ok()?],
            "MultiPolygon" => serde_json::from_value(geometry["coordinates"].clone()).ok()?,
            _ => return None,
        };
        if polygons.is_empty()
            || polygons
                .iter()
                .any(|polygon| polygon.is_empty() || polygon.iter().any(|ring| ring.len() < 4))
        {
            return None;
        }
        let mut bbox = [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ];
        for point in polygons.iter().flatten().flatten() {
            if !point[0].is_finite() || !point[1].is_finite() {
                return None;
            }
            bbox[0] = bbox[0].min(point[0]);
            bbox[1] = bbox[1].min(point[1]);
            bbox[2] = bbox[2].max(point[0]);
            bbox[3] = bbox[3].max(point[1]);
        }
        Some(Self { polygons, bbox })
    }
    fn contains(&self, point: [f64; 2]) -> bool {
        point[0] >= self.bbox[0]
            && point[0] <= self.bbox[2]
            && point[1] >= self.bbox[1]
            && point[1] <= self.bbox[3]
            && self.polygons.iter().any(|polygon| {
                in_ring(point, &polygon[0])
                    && !polygon.iter().skip(1).any(|hole| in_ring(point, hole))
            })
    }
}
fn in_ring([x, y]: [f64; 2], ring: &[[f64; 2]]) -> bool {
    let mut inside = false;
    for index in 0..ring.len() {
        let [ax, ay] = ring[index];
        let [bx, by] = ring[(index + 1) % ring.len()];
        // Les points situés sur un segment de la limite extérieure sont inclus.
        let cross = (x - ax) * (by - ay) - (y - ay) * (bx - ax);
        if cross.abs() < 1e-10
            && x >= ax.min(bx)
            && x <= ax.max(bx)
            && y >= ay.min(by)
            && y <= ay.max(by)
        {
            return true;
        }
        if (ay > y) != (by > y) && x < (bx - ax) * (y - ay) / (by - ay) + ax {
            inside = !inside;
        }
    }
    inside
}

/// Formule de Haversine ; distance géographique en kilomètres.
fn distance_km(a: [f64; 2], b: [f64; 2]) -> f64 {
    let lat = (b[1] - a[1]).to_radians();
    let lon = (b[0] - a[0]).to_radians();
    let h = (lat / 2.0).sin().powi(2)
        + a[1].to_radians().cos() * b[1].to_radians().cos() * (lon / 2.0).sin().powi(2);
    6371.0088 * 2.0 * h.clamp(0.0, 1.0).sqrt().asin()
}

fn compare(
    commune: Commune,
    geometry: &Value,
    stations: &[Value],
    fetched_at: String,
    now: chrono::DateTime<Utc>,
) -> ApiResult<CommuneFuel> {
    let boundary = Boundary::parse(geometry).ok_or_else(unavailable)?;
    let mut ids = HashSet::new();
    // La distance est à vol d’oiseau depuis le centre approximatif de l’emprise communale.
    // Elle ne correspond ni à une position du visiteur ni à un trajet routier.
    let center = [
        (boundary.bbox[0] + boundary.bbox[2]) / 2.0,
        (boundary.bbox[1] + boundary.bbox[3]) / 2.0,
    ];
    let candidates: Vec<_> = stations
        .iter()
        .filter_map(|station| {
            let lon = station["geom"]["lon"].as_f64()?;
            let lat = station["geom"]["lat"].as_f64()?;
            if !lon.is_finite()
                || !lat.is_finite()
                || !(-180.0..=180.0).contains(&lon)
                || !(-90.0..=90.0).contains(&lat)
                || station["id"].is_null()
                || !ids.insert(station["id"].to_string())
            {
                return None;
            }
            let local = boundary.contains([lon, lat]);
            let distance = distance_km(center, [lon, lat]);
            (local || distance <= 25.0).then_some((station, local, distance))
        })
        .collect();
    let station_count = candidates.iter().filter(|(_, local, _)| *local).count();
    let prices = FUELS
        .iter()
        .map(|(key, label)| {
            let mut minimum: Option<i64> = None;
            let mut winners = Vec::new();
            let mut local_found = false;
            // Les stations locales sont traitées d’abord : un voisin moins cher ne les remplace pas.
            for (station, local, distance) in candidates
                .iter()
                .filter(|(_, local, _)| *local)
                .chain(candidates.iter().filter(|(_, local, _)| !*local))
            {
                if !*local && local_found {
                    continue;
                }
                // La liste officielle des carburants disponibles exclut ruptures et non-distribution.
                if !station["carburants_disponibles"]
                    .as_array()
                    .is_some_and(|fuels| fuels.iter().any(|fuel| fuel.as_str() == Some(*label)))
                {
                    continue;
                }
                let Some(price) = station[format!("{key}_prix")]
                    .as_f64()
                    .filter(|price| price.is_finite() && *price > 0.0 && *price < 20.0)
                else {
                    continue;
                };
                let Some(date) = station[format!("{key}_maj")].as_str() else {
                    continue;
                };
                let Ok(updated) = chrono::DateTime::parse_from_rfc3339(date) else {
                    continue;
                };
                if updated > now + Duration::days(1) {
                    continue;
                }
                if *local {
                    local_found = true;
                }
                let milli = (price * 1000.0).round() as i64;
                if minimum.is_none_or(|best| milli < best) {
                    minimum = Some(milli);
                    winners.clear();
                }
                if minimum == Some(milli) {
                    winners.push(StationPrice {
                        id: station["id"]
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| station["id"].to_string()),
                        address: station["adresse"]
                            .as_str()
                            .unwrap_or("Adresse non renseignée")
                            .into(),
                        city: station["ville"].as_str().unwrap_or(&commune.name).into(),
                        updated_at: date.into(),
                        older_than_week: now.signed_duration_since(updated).num_days() >= 7,
                        distance_km: (!*local).then_some((distance * 10.0).round() / 10.0),
                    });
                }
            }
            winners.sort_by(|a, b| a.address.cmp(&b.address).then(a.id.cmp(&b.id)));
            FuelMinimum {
                fuel: (*label).into(),
                search_scope: if local_found {
                    "commune"
                } else if minimum.is_some() {
                    "nearby"
                } else {
                    "none"
                }
                .into(),
                price_per_litre: minimum.map(|value| value as f64 / 1000.0),
                stations: winners,
            }
        })
        .collect();
    Ok(CommuneFuel {
        commune_id: commune.id,
        commune_name: commune.name,
        station_count,
        nearby_radius_km: 25.0,
        prices,
        fetched_at,
        source_url: SOURCE_URL,
    })
}

pub async fn cheapest(
    State(database): State<Database>,
    Path(code): Path<String>,
) -> ApiResult<Json<CommuneFuel>> {
    if code.len() != 5 || !code.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(invalid("Code de commune invalide."));
    }
    let commune = database
        .collection::<Commune>("communes")
        .find_one(doc! {"id":&code})
        .await?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Commune introuvable.".into()))?;
    let geometry_key = format!("fuel-geometry-{code}");
    let (national, geo) = tokio::try_join!(
        cached(&database, NATIONAL_KEY, None),
        cached(&database, &geometry_key, Some(&code))
    )?;
    Ok(Json(compare(
        commune,
        &geo.data["geometry"],
        national.data.as_array().ok_or_else(unavailable)?,
        national.fetched_at,
        Utc::now(),
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn geometry() -> Value {
        json!({"type":"Polygon","coordinates":[[[0,0],[10,0],[10,10],[0,10],[0,0]],[[3,3],[7,3],[7,7],[3,7],[3,3]]]})
    }
    #[test]
    fn contours_excluent_voisins_et_enclaves_et_acceptent_multipolygones() {
        let boundary = Boundary::parse(&geometry()).unwrap();
        assert!(boundary.contains([1.0, 1.0]));
        assert!(boundary.contains([0.0, 2.0]));
        assert!(!boundary.contains([5.0, 5.0]));
        assert!(!boundary.contains([11.0, 1.0]));
        let multi = json!({"type":"MultiPolygon","coordinates":[geometry()["coordinates"].clone(),[[[20,20],[21,20],[21,21],[20,21],[20,20]]]]});
        assert!(Boundary::parse(&multi).unwrap().contains([20.5, 20.5]));
    }
    #[test]
    fn minima_par_carburant_excluent_ruptures_et_prix_invalides_sans_perdre_les_ex_aequo() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-31T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let station = |id: i32, x: f64, price: f64, available: bool| json!({"id":id,"adresse":format!("Station {id}"),"ville":"Test","geom":{"lon":x,"lat":1},"carburants_disponibles":if available {vec!["Gazole"]} else {vec![]},"gazole_prix":price,"gazole_maj":"2026-10-20T12:00:00Z"});
        let mut records = vec![
            station(1, 1.0, 1.8, true),
            station(2, 2.0, 1.8, true),
            station(3, 11.0, 1.0, true),
            station(4, 1.0, 0.5, false),
            station(5, 1.0, -1.0, true),
        ];
        records.push(records[0].clone());
        let commune = Commune {
            id: "00001".into(),
            name: "Test".into(),
            department: "Test".into(),
            region: "Test".into(),
        };
        let result = compare(commune, &geometry(), &records, now.to_rfc3339(), now).unwrap();
        assert_eq!(result.station_count, 4);
        assert_eq!(result.prices[0].price_per_litre, Some(1.8));
        assert_eq!(result.prices[0].stations.len(), 2);
        assert!(result.prices[0].stations[0].older_than_week);
        assert!(result.prices[1].price_per_litre.is_none());
    }
    #[test]
    fn proximite_complete_seulement_les_carburants_absents_et_respecte_25_km() {
        let now = Utc::now();
        let geometry =
            json!({"type":"Polygon","coordinates":[[[0,0],[0.01,0],[0.01,0.01],[0,0.01],[0,0]]]});
        let station = |id: i32, lon: f64, gazole: f64, sp95: Option<f64>| {
            json!({
            "id":id,"adresse":format!("Station {id}"),"ville":"Voisine",
            "geom":{"lon":lon,"lat":0.005},"carburants_disponibles":["Gazole","SP95"],
            "gazole_prix":gazole,"gazole_maj":now.to_rfc3339(),
            "sp95_prix":sp95,"sp95_maj":now.to_rfc3339()})
        };
        let records = vec![
            station(1, 0.005, 2.0, None),
            station(2, 0.1, 1.8, Some(1.9)),
            station(3, 0.15, 1.7, Some(1.9)),
            station(4, 0.3, 1.0, Some(1.0)),
        ];
        let commune = Commune {
            id: "00001".into(),
            name: "Test".into(),
            department: "Test".into(),
            region: "Test".into(),
        };
        let result = compare(commune, &geometry, &records, now.to_rfc3339(), now).unwrap();
        assert_eq!(result.station_count, 1);
        assert_eq!(result.prices[0].price_per_litre, Some(2.0));
        assert_eq!(result.prices[0].search_scope, "commune");
        assert_eq!(result.prices[0].stations[0].distance_km, None);
        assert_eq!(result.prices[1].price_per_litre, Some(1.9));
        assert_eq!(result.prices[1].search_scope, "nearby");
        assert_eq!(result.prices[1].stations.len(), 2);
        assert!(
            result.prices[1]
                .stations
                .iter()
                .all(|s| s.distance_km.is_some_and(|d| d > 0.0 && d <= 25.0))
        );
        assert_eq!(result.prices[2].search_scope, "none");
        let empty_commune = Commune {
            id: "00001".into(),
            name: "Test".into(),
            department: "Test".into(),
            region: "Test".into(),
        };
        let nearby_only = compare(
            empty_commune,
            &geometry,
            &records[1..],
            now.to_rfc3339(),
            now,
        )
        .unwrap();
        assert_eq!(nearby_only.station_count, 0);
        assert_eq!(nearby_only.prices[0].price_per_litre, Some(1.7));
        assert_eq!(nearby_only.prices[0].search_scope, "nearby");
        assert!((distance_km([0.0, 0.0], [0.0, 1.0]) - 111.195).abs() < 0.01);
    }
}
