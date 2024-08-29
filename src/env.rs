use lazy_static::lazy_static;
use std::env::var;

pub struct Env {
    pub cookie_path: String,
    pub cookie_secret: String,
    pub public_url: String,
    pub container_url: String,

    // oauth
    pub oauth_app_id: String,
    pub oauth_app_secret: String,
    pub oauth_server: String,
}

fn get_env() -> Env {
    Env {
        cookie_path: var("COOKIE_PATH").expect("COOKIE_PATH"),
        cookie_secret: var("COOKIE_SECRET").expect("COOKIE_SECRET"),
        public_url: var("PUBLIC_URL").expect("PUBLIC_URL"),
        container_url: var("CONTAINER_URL").expect("CONTAINER_URL"),

        oauth_app_id: var("OAUTH_APP_ID").expect("OAUTH_APP_ID"),
        oauth_app_secret: var("OAUTH_APP_SECRET").expect("OAUTH_APP_SECRET"),
        oauth_server: var("OAUTH_SERVER").expect("OAUTH_SERVER"),
    }
}

lazy_static! {
    pub static ref ENV: Env = get_env();
}
