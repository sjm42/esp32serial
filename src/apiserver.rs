// apiserver.rs

use axum::body::Body;
use axum::http::{header, Response};
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, response::Html, routing::*, Json, Router};
pub use axum_macros::debug_handler;
use std::{net, net::SocketAddr, pin::Pin, sync::Arc};
use askama::Template;
use tokio::time::{sleep, Duration};

// use tower_http::trace::TraceLayer;

use crate::*;

pub async fn run_api_server(state: Arc<Pin<Box<MyState>>>) -> anyhow::Result<()> {
    loop {
        if *state.wifi_up.read().await {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    let listen = format!("0.0.0.0:{}", state.config.port);
    let addr = listen.parse::<SocketAddr>()?;

    let app = Router::new()
        .route("/", get(get_index))
        .route("/favicon.ico", get(get_favicon))
        .route("/form.js", get(get_formjs))
        .route("/conf", get(get_conf).post(set_conf).options(options))
        .route("/reset_conf", get(reset_conf))
        .with_state(state);
    // .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening to {listen}");
    Ok(axum::serve(listener, app.into_make_service()).await?)
}

pub async fn options(State(state): State<Arc<Pin<Box<MyState>>>>) -> Response<Body> {
    let cnt = {
        let mut c = state.api_cnt.write().await;
        *c += 1;
        *c
    };
    info!("#{cnt} options()");

    (
        StatusCode::OK,
        [
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::ACCESS_CONTROL_ALLOW_METHODS, "get,post"),
            (header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type"),
        ],
    )
        .into_response()
}


pub async fn get_index(State(state): State<Arc<Pin<Box<MyState>>>>) -> Response<Body> {
    let cnt = {
        let mut c = state.api_cnt.write().await;
        *c += 1;
        *c
    };
    info!("#{cnt} get_index()");

    let index = match state.config.clone().render() {
        Err(e) => {
            let err_msg = format!("Index template error: {e:?}\n");
            error!("{err_msg}");
            return (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response();
        }
        Ok(s) => s,
    };
    (StatusCode::OK, Html(index)).into_response()
}

pub async fn get_favicon(State(state): State<Arc<Pin<Box<MyState>>>>) -> Response<Body> {
    let cnt = {
        let mut c = state.api_cnt.write().await;
        *c += 1;
        *c
    };
    info!("#{cnt} get_favicon()");

    let favicon = include_bytes!("favicon.ico");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/vnd.microsoft.icon")],
        favicon.to_vec(),
    )
        .into_response()
}

pub async fn get_formjs(State(state): State<Arc<Pin<Box<MyState>>>>) -> Response<Body> {
    let cnt = {
        let mut c = state.api_cnt.write().await;
        *c += 1;
        *c
    };
    info!("#{cnt} get_formjs()");

    let formjs = include_bytes!("form.js");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        formjs.to_vec(),
    )
        .into_response()
}
pub async fn get_conf(State(state): State<Arc<Pin<Box<MyState>>>>) -> (StatusCode, Json<MyConfig>) {
    let cnt = {
        let mut c = state.api_cnt.write().await;
        *c += 1;
        *c
    };
    info!("#{cnt} get_conf()");

    (StatusCode::OK, Json(state.config.clone()))
}

pub async fn set_conf(
    State(state): State<Arc<Pin<Box<MyState>>>>,
    Json(mut config): Json<MyConfig>,
) -> (StatusCode, String) {
    let cnt = {
        let mut c = state.api_cnt.write().await;
        *c += 1;
        *c
    };
    info!("#{cnt} set_conf()");

    if config.v4mask > 30 {
        let msg = "IPv4 mask error: bits must be between 0..30";
        error!("{}", msg);
        return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string());
    }

    if config.v4dhcp {
        // clear out these if we are using DHCP
        config.v4addr = net::Ipv4Addr::new(0, 0, 0, 0);
        config.v4mask = 0;
        config.v4gw = net::Ipv4Addr::new(0, 0, 0, 0);
        config.dns1 = net::Ipv4Addr::new(0, 0, 0, 0);
        config.dns2 = net::Ipv4Addr::new(0, 0, 0, 0);
    }

    info!("Saving new config to nvs...");
    Box::pin(save_conf(state, config)).await
}

pub async fn reset_conf(State(state): State<Arc<Pin<Box<MyState>>>>) -> (StatusCode, String) {
    let cnt = {
        let mut c = state.api_cnt.write().await;
        *c += 1;
        *c
    };
    info!("#{cnt} reset_conf()");

    info!("Saving  default config to nvs...");
    Box::pin(save_conf(state, MyConfig::default())).await
}

async fn save_conf(state: Arc<Pin<Box<MyState>>>, config: MyConfig) -> (StatusCode, String) {
    let mut nvs = state.nvs.write().await;
    match config.to_nvs(&mut nvs) {
        Ok(_) => {
            info!("Config saved to nvs. Restarting soon...");
            // schedule a restart
            *state.restart.write().await = true;
            (StatusCode::OK, "OK".to_string())
        }
        Err(e) => {
            let msg = format!("Nvs write error: {e:?}");
            error!("{}", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    }
}
// EOF
