use actix_service::Service;
use actix_session::{storage::CookieSessionStore, SessionExt, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpMessage, HttpServer};
use dotenv::dotenv;
use log::*;
use oauth2::{
    basic::BasicClient, AuthType, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
use openvscode_farm::{
    env::ENV,
    login::LoginState,
    routes::{self},
    LOGIN_SESSION_KEY,
};
use ring::digest;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("Bootstraping...");

    let secret = ENV.cookie_secret.clone();
    let secret = digest::digest(&digest::SHA512, secret.as_bytes());
    let gitlab_client_id = ClientId::new(ENV.oauth_app_id.clone());
    let gitlab_client_secret = ClientSecret::new(ENV.oauth_app_secret.clone());
    let auth_url = AuthUrl::new(format!("{}/api/authorize", ENV.oauth_server))
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(format!("{}/api/token", ENV.oauth_server))
        .expect("Invalid token endpoint URL");
    let redirect_uri = format!("{}/callback", ENV.public_url);
    info!("Gitlab application redirect uri is {}", redirect_uri);
    let client = BasicClient::new(
        gitlab_client_id,
        Some(gitlab_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri).expect("Invalid redirect URL"))
    .set_auth_type(AuthType::RequestBody); // YXPortal only supports RequestBody type auth

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap_fn(|req, srv| {
                let sess = req.get_session();
                let sess_login = sess.get::<LoginState>(LOGIN_SESSION_KEY);
                let current_state = sess_login.ok().flatten();

                if let Some(cs) = current_state {
                    debug!("Login state extracted: {:?}", cs);
                    req.extensions_mut().insert(cs);
                }

                srv.call(req)
            })
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(secret.as_ref()),
                )
                .cookie_path(ENV.cookie_path.clone())
                .cookie_secure(true)
                .cookie_http_only(true)
                .build(),
            )
            .wrap(middleware::Logger::new(
                r#"%a %{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#, // add real ip for reverse proxy
            ))
            .service(
                web::scope("/vscode")
                    .route("/login", web::get().to(routes::login))
                    .route("/callback", web::get().to(routes::callback))
                    .route("/start", web::get().to(routes::start)),
            )
    })
    .bind("127.0.0.1:3030")?
    .run()
    .await?;
    Ok(())
}
