use sportsday_scoreboard_v2::{self as app, *};
use std::sync::atomic::{AtomicU64, Ordering};

// Helper function to create a unique test database path
fn get_test_db_path() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::fs::create_dir_all("./test").ok();
    let path = format!("./test/integration_test_{}_{}.db", timestamp, id);
    // Remove the file if it exists
    std::fs::remove_file(&path).ok();
    path
}

#[actix_web::test]
async fn test_database_operations() {
    let pool = async_sqlite::PoolBuilder::new()
        .path(&get_test_db_path())
        .open()
        .await
        .unwrap();

    app::create_tables(&pool).await.unwrap();

    // Test user creation
    let user = db::users::Users::new("test@example.com".to_string(), true, false);
    user.insert(&pool).await.unwrap();

    let users = db::users::Users::all(&pool).await.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].email, "test@example.com");

    // Test year creation
    let year = db::years::Years::new("2024".to_string(), "Year 2024".to_string());
    year.insert(&pool).await.unwrap();

    let years = db::years::Years::all(&pool).await.unwrap();
    assert_eq!(years.len(), 1);

    // Test event creation
    let event = db::events::Events::new(
        "2024-mixed-test".to_string(),
        "Test Event".to_string(),
        "2024".to_string(),
        "mixed".to_string(),
        "test".to_string(),
        "{}".to_string(),
    );
    event.insert(&pool).await.unwrap();

    let events = db::events::Events::all(&pool).await.unwrap();
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn test_configuration_loading_and_plan_building() {
    use configurator::build::build_plan;
    use configurator::parser::{ApplicabilityRules, Configuration, Event, Form, Score, Year};
    use configurator::run::run;

    let config = Configuration {
        version: "1.0.0".to_string(),
        genders: vec!["boys".to_string(), "girls".to_string()],
        scores: vec![Score {
            name: "1st".to_string(),
            value: 10,
            default: true,
        }],
        years: vec![Year {
            id: "year7".to_string(),
            name: "Year 7".to_string(),
        }],
        forms: vec![Form {
            id: "form1".to_string(),
            name: "Form 1".to_string(),
            colour: "#ff0000".to_string(),
        }],
        events: vec![Event {
            id: "event1".to_string(),
            name: "Event 1".to_string(),
            applicable_years: ApplicabilityRules::All,
            applicable_genders: ApplicabilityRules::All,
        }],
    };

    let plan = build_plan(config);
    assert_eq!(plan.year_plans.len(), 1);
    assert_eq!(plan.year_plans[0].events.len(), 2); // boys + girls

    // Test running the plan
    let pool = async_sqlite::PoolBuilder::new()
        .path(&get_test_db_path())
        .open()
        .await
        .unwrap();

    app::create_tables(&pool).await.unwrap();
    run(plan, &pool).await.unwrap();

    let years = db::years::Years::all(&pool).await.unwrap();
    assert_eq!(years.len(), 1);

    let events = db::events::Events::all(&pool).await.unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn test_user_session_flow() {
    let pool = async_sqlite::PoolBuilder::new()
        .path(&get_test_db_path())
        .open()
        .await
        .unwrap();

    app::create_tables(&pool).await.unwrap();

    // Create a user
    let user = db::users::Users::get_or_create("test@example.com".to_string(), &pool)
        .await
        .unwrap();

    assert!(user.id.is_some());
    assert_eq!(user.email, "test@example.com");

    // Create a session
    let session = user.new_session();
    session.clone().insert(&pool).await.unwrap();

    // Verify the session
    let verified = db::user_sessions::UserSessions::verify(&pool, session.id.clone())
        .await
        .unwrap();

    assert!(verified.verified);
    assert_eq!(verified._id, session.id);
}

#[tokio::test]
async fn test_logger_functionality() {
    use log::Level;
    use logger::LogCollector;

    let collector = LogCollector::new(10);

    collector.add_entry(Level::Info, "Test message 1", Some("test_module"));
    collector.add_entry(Level::Error, "Test message 2", Some("test_module"));

    let entries = collector.get_entries();
    assert_eq!(entries.len(), 2);

    collector.clear();
    assert_eq!(collector.get_entries().len(), 0);
}

#[tokio::test]
async fn test_ternary_macro() {
    let result = ternary!(true => "yes", "no");
    assert_eq!(result, "yes");

    let result = ternary!(false => "yes", "no");
    assert_eq!(result, "no");

    let result = ternary!(5 > 3 => 100, 200);
    assert_eq!(result, 100);
}
