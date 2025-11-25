use actix_web::{
    cookie::{time::Duration, Cookie},
    get,
    web::{self, Redirect},
    HttpRequest, HttpResponse,
};
use askama::Template;
use log::{debug, error};
use reqwest::StatusCode;

use crate::{db, templates::IndexTemplate, AppState};

// https://github.com/login/oauth/authorize?client_id=Ov23liSm3b4ovVlirQNU&redirect_uri=http://127.0.0.1:3000/oauth/callback&scope=user:email

#[get("/oauth/callback")]
pub async fn callback_get(
    state: web::Data<AppState>,
    req: HttpRequest,
    params: web::Query<CallbackParams>,
) -> HttpResponse {
    let exchage_req = state
        .client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&GithubAccessTokenBody {
                client_id: state.oauth_creds.client_id.clone(),
                client_secret: state.oauth_creds.client_secret.clone(),
                code: params.code.clone(),
            })
            .unwrap(),
        )
        .send()
        .await
        .unwrap();
    if exchage_req.status() != StatusCode::OK {
        error!(
            "Github Token Exchange errored with code {} and error {}",
            exchage_req.status(),
            exchage_req.text().await.unwrap()
        );
        return HttpResponse::InternalServerError()
            .body("Eror exchanging Github Access Token for auth");
    }

    let exchange_req_json = exchage_req.json::<GithubAccessCodeResBody>().await.unwrap();
    let emails_req = state
        .client
        .get("https://api.github.com/user/emails")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!("Bearer {}", exchange_req_json.access_token),
        )
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .unwrap();
    if emails_req.status() != StatusCode::OK {
        error!(
            "Github Email Retrieval errored with code {} and error {}",
            emails_req.status(),
            emails_req.text().await.unwrap()
        );
        return HttpResponse::InternalServerError().body("Eror retrieving emails from Github");
    }
    let emails_req_json = emails_req
        .json::<Vec<GithubUserEmailsResBody>>()
        .await
        .unwrap();

    // We want to loop through and find a college email if possible, otherwise faillback to the first
    let user_emails = emails_req_json
        .iter()
        .filter(|email| email.verified == true)
        .cloned()
        .collect::<Vec<GithubUserEmailsResBody>>();
    let mut user_email = user_emails[0].email.clone();
    for email in user_emails.iter() {
        if email.email.ends_with("utcsheffield.org.uk") {
            user_email = email.email.clone();
        }
    }
    let user = db::users::Users::get_or_create(user_email, &state.pool)
        .await
        .unwrap();

    debug!("Got User with ID {}", user.id.unwrap());
    let session = user.clone().new_session();
    debug!(
        "Created session for user {} with id {}",
        user.id.unwrap(),
        session.id
    );

    session.clone().insert(&state.pool).await.unwrap();

    let cookie = Cookie::build("session_data", session.clone().id)
        .path("/") // Make cookie available for all paths
        .max_age(Duration::days(10))
        .http_only(true) // Prevent JavaScript access for security
        .finish();

    debug!("Setting cookie: session_data={}", session.id);

    let redirect_to = req
        .cookie("redirect-to")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| "/".to_string());
    HttpResponse::Found()
        .append_header(("Location", redirect_to))
        .cookie(cookie)
        .finish()
}

#[derive(serde::Deserialize)]
struct CallbackParams {
    code: String,
}

#[derive(serde::Serialize)]
struct GithubAccessTokenBody {
    client_id: String,
    client_secret: String,
    code: String,
}

#[derive(serde::Deserialize)]
struct GithubAccessCodeResBody {
    access_token: String,
}

#[derive(serde::Deserialize, Clone)]
struct GithubUserEmailsResBody {
    email: String,
    verified: bool,
}
