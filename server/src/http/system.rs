use crate::http::error::CustomError;
use crate::http::mapper;
use crate::streaming::systems::system::System;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use iggy::models::client_info::{ClientInfo, ClientInfoDetails};
use iggy::models::stats::Stats;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::auth;

const NAME: &str = "Iggy HTTP";
const PONG: &str = "pong";

pub fn router(system: Arc<RwLock<System>>) -> Router {
    Router::new()
        .route("/", get(|| async { NAME }))
        .route("/ping", get(|| async { PONG }))
        .route("/stats", get(get_stats))
        .route("/clients", get(get_clients))
        .route("/clients/:client_id", get(get_client))
        .with_state(system)
}

async fn get_stats(State(system): State<Arc<RwLock<System>>>) -> Result<Json<Stats>, CustomError> {
    let user_id = auth::resolve_user_id();
    let system = system.read().await;
    system.permissioner.get_stats(user_id)?;
    let stats = system.get_stats().await;
    Ok(Json(stats))
}

async fn get_client(
    State(system): State<Arc<RwLock<System>>>,
    Path(client_id): Path<u32>,
) -> Result<Json<ClientInfoDetails>, CustomError> {
    let user_id = auth::resolve_user_id();
    let system = system.read().await;
    system.permissioner.get_client(user_id)?;
    let client = system.get_client(client_id).await?;
    let client = client.read().await;
    let client = mapper::map_client(&client).await;
    Ok(Json(client))
}

async fn get_clients(
    State(system): State<Arc<RwLock<System>>>,
) -> Result<Json<Vec<ClientInfo>>, CustomError> {
    let user_id = auth::resolve_user_id();
    let system = system.read().await;
    system.permissioner.get_clients(user_id)?;
    let clients = system.get_clients().await;
    let clients = mapper::map_clients(&clients).await;
    Ok(Json(clients))
}
