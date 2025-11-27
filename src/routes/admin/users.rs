use actix_web::{get, post, web, HttpResponse};
use askama::Template;

use crate::{
    db,
    templates::{AdminUsersEditTemplate, AdminUsersListTemplate, AdminUsersNewTemplate},
    ternary, AppState,
};

#[get("")]
pub async fn list(state: web::Data<AppState>) -> HttpResponse {
    let users = db::users::Users::all(&state.pool).await.unwrap();

    HttpResponse::Ok().body(
        AdminUsersListTemplate { users }
            .render()
            .expect("Template should be valid"),
    )
}

#[get("/new")]
pub async fn new(_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body(
        AdminUsersNewTemplate {}
            .render()
            .expect("Template should be valid"),
    )
}

#[post("")]
pub async fn create(state: web::Data<AppState>, params: web::Form<UpdateProps>) -> HttpResponse {
    db::users::Users::new(
        params.email.clone(),
        ternary!(params.has_admin == Some("on".to_string()) => true, false),
        ternary!(params.has_set_score  == Some("on".to_string()) => true, false),
    )
    .insert(&state.pool)
    .await
    .unwrap();
    HttpResponse::Found()
        .append_header(("Location", "/admin/users"))
        .finish()
}

#[get("/edit/{id}")]
pub async fn edit(state: web::Data<AppState>, params: web::Path<PathProps>) -> HttpResponse {
    let user = db::users::Users::find_by_id(params.id, &state.pool)
        .await
        .unwrap()
        .unwrap();

    HttpResponse::Ok().body(
        AdminUsersEditTemplate { user }
            .render()
            .expect("template should be valid"),
    )
}

#[post("/edit/{id}")]
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<PathProps>,
    body: web::Form<UpdateProps>,
) -> HttpResponse {
    db::users::Users::update(
        &state.pool,
        path.id,
        body.email.clone(),
        ternary!(body.has_admin == Some("on".to_string()) => true, false),
        ternary!(body.has_set_score  == Some("on".to_string()) => true, false),
    )
    .await
    .unwrap();

    HttpResponse::Found()
        .append_header(("Location", "/admin/users"))
        .finish()
}

#[derive(serde::Deserialize)]
struct UpdateProps {
    email: String,
    has_admin: Option<String>,
    has_set_score: Option<String>,
}

#[derive(serde::Deserialize)]
struct PathProps {
    id: i64,
}
