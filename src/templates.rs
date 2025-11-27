use askama::Template;
use std::collections::HashMap;

use crate::{
    configurator::{
        self,
        parser::{Form, Score},
    },
    db::{events::Events, users::Users, years::Years},
    routes::results::ResultsEvent,
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

#[derive(Template)]
#[template(path = "scoreboard.html")]
pub struct ScoreboardTemplate {
    pub scores: String,
}

#[derive(Template)]
#[template(path = "partials/scoreboard.html")]
pub struct ScoreboardPartialTemplate {
    pub forms: Vec<Form>,
    pub years: Vec<Years>,
    pub scores: HashMap<String, HashMap<String, i64>>,
    pub year_totals: HashMap<String, i64>,
    pub form_totals: HashMap<String, i64>,
    pub grand_total: i64,
}

#[derive(Template)]
#[template(path = "set_scores.html")]
pub struct SetScoresTemplate {
    pub events: Vec<Events>,
    pub activity_types: Vec<configurator::parser::Event>,
    pub year_types: Vec<configurator::parser::Year>,
    pub group_types: Vec<String>,
    pub forms: Vec<Form>,
    pub scores: Vec<Score>,
}

#[derive(Template)]
#[template(path = "results.html")]
pub struct ResultsTemplate {
    pub forms: Vec<Form>,
    pub events: Vec<ResultsEvent>,
}

#[derive(Template)]
#[template(path = "admin/index.html")]
pub struct AdminIndexTemplate {}

#[derive(Template)]
#[template(path = "admin/users/list.html")]
pub struct AdminUsersListTemplate {
    pub users: Vec<Users>,
}

#[derive(Template)]
#[template(path = "admin/users/new.html")]
pub struct AdminUsersNewTemplate {}

#[derive(Template)]
#[template(path = "admin/users/edit.html")]
pub struct AdminUsersEditTemplate {
    pub user: Users,
}
