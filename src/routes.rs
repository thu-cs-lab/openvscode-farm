use actix_http::{header, Method};
use actix_session::Session;
use actix_web::{
    web::{Data, Query},
    HttpResponse, Result,
};
use log::{info, warn};
use oauth2::{
    basic::BasicClient, http::HeaderMap, reqwest::async_http_client, AuthorizationCode, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Url;
use serde::Deserialize;
use tokio::process::Command;

use crate::{env::ENV, err, login::LoginState, LOGIN_SESSION_KEY};

pub async fn login(session: Session, data: Data<BasicClient>) -> Result<HttpResponse> {
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token) = &data
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("api".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    session.insert("code", csrf_token.secret())?;
    session.insert("pkce", pkce_code_verifier.secret())?;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish())
}

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

pub async fn callback(
    session: Session,
    data: Data<BasicClient>,
    params: Query<AuthRequest>,
) -> Result<HttpResponse> {
    let code = AuthorizationCode::new(params.code.clone());
    let expected = match session.get::<String>("code") {
        Ok(Some(e)) => e,
        _ => return Ok(HttpResponse::BadRequest().finish()),
    };
    let pkce = match session.get::<String>("pkce") {
        Ok(Some(e)) => e,
        _ => return Ok(HttpResponse::BadRequest().finish()),
    };

    // Code has been extracted, we should remove it from the session now
    session.remove("code");
    session.remove("login_id");

    // Check code
    if expected != params.state {
        return Ok(HttpResponse::Found()
            .append_header((header::LOCATION, "/"))
            .finish());
    }

    let pkce_verifier = PkceCodeVerifier::new(pkce);
    match &data
        .exchange_code(code)
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
    {
        Ok(token) => {
            let access_token = token.access_token().secret();
            info!("Got token {}", access_token);
            let user = read_user(access_token).await?;

            info!(
                "User {}({},{}) logged in",
                user.user_name, user.real_name, user.student_id
            );

            let state = LoginState {
                user_name: user.user_name.clone(),
            };

            session.insert(LOGIN_SESSION_KEY, state)?;

            Ok(HttpResponse::Found()
                .append_header((header::LOCATION, format!("{}/start", ENV.public_url)))
                .finish())
        }
        Err(err) => {
            warn!("Got error {:?} when login", err);
            Ok(HttpResponse::Found()
                .append_header((header::LOCATION, "/"))
                .finish())
        }
    }
}

async fn read_user(access_token: &str) -> Result<UserInfo> {
    let url = Url::parse(&format!("{}/api/self", ENV.oauth_server)).map_err(err)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", access_token).parse().map_err(err)?,
    );
    let resp = async_http_client(oauth2::HttpRequest {
        url,
        method: Method::GET,
        headers,
        body: Vec::new(),
    })
    .await
    .map_err(err)?;
    let user: UserInfo = serde_json::from_slice(&resp.body).map_err(err)?;
    Ok(user)
}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    user_name: String,
    real_name: String,
    student_id: String,
}

fn generate_secret_token() -> String {
    let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    rand_string
}

pub async fn start(login: LoginState) -> Result<HttpResponse> {
    // Use user_id to create a unique container name
    Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg(format!("vscs-{}", login.user_name))
        .arg("--init")
        .arg("--entrypoint")
        .arg("")
        .arg("-p")
        .arg("3000")
        .arg(&ENV.image_name)
        .arg("sh")
        .arg("-c")
        .arg("exec ${OPENVSCODE_SERVER_ROOT}/bin/openvscode-server \"${@}\"")
        .arg("--")
        .arg("--connection-token")
        .arg(generate_secret_token())
        .arg("--host")
        .arg("0.0.0.0")
        .arg("--enable-remote-auto-shutdown")
        .output()
        .await
        .map_err(err)?;
    // Start the container
    Command::new("docker")
        .arg("start")
        .arg(format!("vscs-{}", login.user_name))
        .output()
        .await
        .map_err(err)?;
    // Now, use docker inspect to get the container port and secret
    // docker inspect -f '{{(index (index .NetworkSettings.Ports "3000/tcp") 0).HostPort}}' vscs-<user_id>
    let output = Command::new("docker")
    .arg("inspect")
    .arg(format!("vscs-{}", login.user_name))
    .arg("-f")
    .arg("{{(index (index .NetworkSettings.Ports \"3000/tcp\") 0).HostPort}} {{ index (index .Config.Cmd) 5 }}")
    .output()
    .await
    .map_err(err)?;
    let output = String::from_utf8(output.stdout).map_err(err)?;
    let parts: Vec<&str> = output.trim().split(' ').collect();
    if parts.len() != 2 {
        return Err(err("Failed to parse output").into());
    }
    let port = parts[0];
    let secret = parts[1];
    let url = ENV
        .container_url
        .clone()
        .replace("{port}", port)
        .replace("{token}", secret);
    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, url))
        .finish())
}
