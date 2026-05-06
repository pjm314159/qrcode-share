use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::state::WechatTokenCache;

#[derive(Debug, Deserialize)]
pub struct JsapiTicketRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct JsapiTicketResponse {
    pub app_id: String,
    pub timestamp: i64,
    pub nonce_str: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct WechatErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct WechatStatusResponse {
    pub available: bool,
    pub reason: Option<String>,
}

pub async fn get_wechat_status(
    State(state): State<AppState>,
) -> Json<WechatStatusResponse> {
    let status = state.wechat_status.read();
    Json(WechatStatusResponse {
        available: status.available,
        reason: status.reason.clone(),
    })
}

pub async fn get_jsapi_ticket(
    State(state): State<AppState>,
    Json(body): Json<JsapiTicketRequest>,
) -> Result<Json<JsapiTicketResponse>, (StatusCode, Json<WechatErrorResponse>)> {
    {
        let status = state.wechat_status.read();
        if !status.available {
            return Err(wechat_error(&format!(
                "WeChat JS-SDK not available: {}",
                status.reason.as_deref().unwrap_or("Unknown reason")
            )));
        }
    }

    let config = &state.config;

    let appid = config
        .wx_appid
        .as_ref()
        .ok_or_else(|| wechat_error("WeChat AppID not configured"))?;
    let secret = config
        .wx_secret
        .as_ref()
        .ok_or_else(|| wechat_error("WeChat Secret not configured"))?;

    let cache = get_or_refresh_token(appid, secret, &state).await?;

    let timestamp = chrono::Utc::now().timestamp();
    let nonce_str = uuid::Uuid::new_v4().to_string().replace('-', "");

    let sign_str = format!(
        "jsapi_ticket={}&noncestr={}&timestamp={}&url={}",
        cache.jsapi_ticket, nonce_str, timestamp, body.url
    );

    let signature = sha1_hash(&sign_str);

    Ok(Json(JsapiTicketResponse {
        app_id: appid.clone(),
        timestamp,
        nonce_str,
        signature,
    }))
}

async fn get_or_refresh_token(
    appid: &str,
    secret: &str,
    state: &AppState,
) -> Result<WechatTokenCache, (StatusCode, Json<WechatErrorResponse>)> {
    {
        let cache = state.wechat_token_cache.read();
        if let Some(ref c) = *cache {
            let now = chrono::Utc::now();
            if now < c.expires_at && now < c.ticket_expires_at {
                return Ok(c.clone());
            }
        }
    }

    let access_token = fetch_access_token(appid, secret).await?;
    let jsapi_ticket = fetch_jsapi_ticket(&access_token).await?;

    let now = chrono::Utc::now();
    let cache = WechatTokenCache {
        access_token,
        expires_at: now + chrono::Duration::seconds(7000),
        jsapi_ticket,
        ticket_expires_at: now + chrono::Duration::seconds(7000),
    };

    {
        let mut guard = state.wechat_token_cache.write();
        *guard = Some(cache.clone());
    }

    Ok(cache)
}

async fn fetch_access_token(
    appid: &str,
    secret: &str,
) -> Result<String, (StatusCode, Json<WechatErrorResponse>)> {
    let url = format!(
        "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
        appid, secret
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| wechat_error(&format!("Failed to fetch access_token: {}", e)))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| wechat_error(&format!("Failed to parse access_token response: {}", e)))?;

    data.get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            let errcode = data.get("errcode").and_then(|v| v.as_i64()).unwrap_or(0);
            let errmsg = data
                .get("errmsg")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            wechat_error(&format!("WeChat API error ({}): {}", errcode, errmsg))
        })
}

async fn fetch_jsapi_ticket(
    access_token: &str,
) -> Result<String, (StatusCode, Json<WechatErrorResponse>)> {
    let url = format!(
        "https://api.weixin.qq.com/cgi-bin/ticket/getticket?access_token={}&type=jsapi",
        access_token
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| wechat_error(&format!("Failed to fetch jsapi_ticket: {}", e)))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| wechat_error(&format!("Failed to parse jsapi_ticket response: {}", e)))?;

    let errcode = data.get("errcode").and_then(|v| v.as_i64()).unwrap_or(-1);
    if errcode != 0 {
        let errmsg = data
            .get("errmsg")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(wechat_error(&format!(
            "WeChat jsapi_ticket error ({}): {}",
            errcode, errmsg
        )));
    }

    data.get("ticket")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| wechat_error("Missing ticket in jsapi_ticket response"))
}

fn sha1_hash(input: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn wechat_error(msg: &str) -> (StatusCode, Json<WechatErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(WechatErrorResponse {
            error: msg.to_string(),
        }),
    )
}
