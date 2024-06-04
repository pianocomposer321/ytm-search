use std::{env, fs, io, path::PathBuf};

use reqwest::Client;
use ytmapi_rs::auth::OAuthToken;

fn get_cache_dir() -> PathBuf {
    dirs::cache_dir().unwrap().join("ytm-search")
}

async fn generate_token() -> Result<OAuthToken, ytmapi_rs::Error> {
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir)?;

    let (code, url) = ytmapi_rs::generate_oauth_code_and_url().await?;
    println!("Go to {url}, finish the login flow, and press enter when done");
    let mut _buf = String::new();
    let _ = io::stdin().read_line(&mut _buf);
    let token = ytmapi_rs::generate_oauth_token(code).await?;
    if let Ok(token_json) = serde_json::to_string(&token) {
        fs::write(&cache_dir.join("oauth.json"), token_json)?;
    }

    Ok(token)
}

#[tokio::main]
pub async fn main() -> Result<(), ytmapi_rs::Error> {
    // NOTE: The token can be re-used until it expires, and refreshed once it has,
    // so it's recommended to save it to a file here.
    let token: OAuthToken;
    if let Ok(contents) = std::fs::read_to_string(&get_cache_dir().join("oauth.json")) {
        let client = Client::new();
        let maybe_expired_token: OAuthToken = serde_json::from_str(&contents).expect("Invalid token json");
        token = maybe_expired_token.refresh(&client).await.unwrap();
    } else {
        token = generate_token().await?;
    }

    let yt = ytmapi_rs::YtMusic::from_oauth_token(token);

    let query = env::args().nth(1).unwrap();
    let result = yt.search_songs(&query).await;
    println!("{}", serde_json::to_string(&result.unwrap()).unwrap());
    Ok(())
}
