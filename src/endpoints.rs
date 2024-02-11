use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::DrossManagerState;
use crate::faery::Faery;

pub async fn list_faeries(State(state): State<Arc<DrossManagerState>>) -> Response {

    let mut res = state.db
        .lock().await
        .query("SELECT * FROM faeries", ())
        .await.unwrap();

    let mut faeries: Vec<Faery> = Vec::new();

    while let Some(row) = res.next().await.unwrap() {
            let faery = Faery {
                name: row.get(0).unwrap(),
                email: row.get(1).unwrap(),
                is_admin: row.get(2).unwrap(),
                auth_token: row.get(3).unwrap(),
                dross: row.get(4).unwrap(),
            };
        faeries.push(faery);
    }
    (StatusCode::OK, Json(faeries)).into_response()
}