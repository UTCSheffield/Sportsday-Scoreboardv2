use async_sqlite::rusqlite::Error as RusqliteError;
use async_sqlite::{rusqlite::Row, Pool};
use log::debug;

use crate::db::user_sessions::UserSessions;
use crate::ternary;

#[derive(Clone, PartialEq, Debug)]
pub struct Users {
    pub id: Option<i64>,
    pub email: String,
    pub has_admin: bool,
    pub has_set_score: bool,
}

impl Users {
    pub fn new(email: String, has_admin: bool, has_set_score: bool) -> Self {
        Self {
            id: None,
            email,
            has_admin,
            has_set_score,
        }
    }
    fn map_from_row(row: &Row) -> Result<Self, RusqliteError> {
        Ok(Self {
            id: row.get(0)?,
            email: row.get(1)?,
            has_admin: ternary!(row.get(2)? => true, false),
            has_set_score: ternary!(row.get(3)? => true, false),
        })
    }

    pub async fn find_by_email(
        email: String,
        pool: &Pool,
    ) -> Result<Option<Self>, async_sqlite::Error> {
        pool.conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, email, has_admin, has_set_score FROM users WHERE email = ?1",
            )?;
            let mut rows = stmt.query([email])?;

            if let Some(row) = rows.next()? {
                Ok(Some(Self::map_from_row(row)?))
            } else {
                Ok(None)
            }
        })
        .await
    }

    pub async fn get_or_create(email: String, pool: &Pool) -> Result<Self, async_sqlite::Error> {
        debug!("Attempting to get or create user with email: {}", email);

        // Try to find existing user
        if let Some(user) = Self::find_by_email(email.clone(), pool).await? {
            debug!("User found with email: {}", user.email);
            return Ok(user);
        }

        // User doesn't exist, create new one
        debug!("User not found, creating new user with email: {}", email);
        let new_user = Self::new(email.clone(), false, false);

        // Insert the user and get the ID
        let user_id = pool
            .conn(move |conn| {
                conn.execute(
                    "INSERT INTO users(email, has_admin, has_set_score) VALUES (?1, ?2, ?3);",
                    [
                        email.clone(),
                        ternary!(new_user.has_admin => 1, 0).to_string(),
                        ternary!(new_user.has_set_score => 1, 0).to_string(),
                    ],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .await?;

        debug!("Created user with id: {}", user_id);

        Ok(Self {
            id: Some(user_id),
            email: new_user.email,
            has_admin: new_user.has_admin,
            has_set_score: new_user.has_set_score,
        })
    }

    pub async fn insert(self, pool: &Pool) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            conn.execute(
                "INSERT INTO users(email, has_admin, has_set_score) VALUES (?1, ?2, ?3);",
                [
                    self.email,
                    ternary!(self.has_admin => 1, 0).to_string(),
                    ternary!(self.has_set_score => 1, 0).to_string(),
                ],
            )
            .unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn all(pool: &Pool) -> Result<Vec<Self>, async_sqlite::Error> {
        pool.conn(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM users")?;
            let event_iter = stmt
                .query_map([], |row| Ok(Self::map_from_row(row).unwrap()))
                .unwrap();
            let mut events = Vec::new();

            for event in event_iter {
                events.push(event?);
            }
            Ok(events)
        })
        .await
    }

    pub async fn find_by_id(id: i64, pool: &Pool) -> Result<Option<Self>, async_sqlite::Error> {
        pool.conn(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM users WHERE id = ?1")?;
            let mut rows = stmt.query([id])?;

            if let Some(row) = rows.next()? {
                Ok(Some(Self::map_from_row(row)?))
            } else {
                Ok(None)
            }
        })
        .await
    }

    pub async fn update(
        pool: &Pool,
        id: i64,
        email: String,
        has_admin: bool,
        has_set_score: bool,
    ) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            conn.execute(
                "UPDATE users SET email = ?1, has_admin = ?2, has_set_score = ?3 WHERE id = ?4;",
                [
                    email,
                    ternary!(has_admin => 1, 0).to_string(),
                    ternary!(has_set_score => 1, 0).to_string(),
                    id.to_string(),
                ],
            )
            .unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub fn new_session(self) -> UserSessions {
        UserSessions::new(self.id.unwrap(), self.has_admin, self.has_set_score)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_harness;

    use super::*;

    #[test]
    fn is_user_constructed_properly() {
        assert_eq!(
            Users::new("example@example.com".to_string(), true, true),
            Users {
                id: None,
                email: "example@example.com".to_string(),
                has_admin: true,
                has_set_score: true
            }
        )
    }

    #[tokio::test]
    async fn find_by_email_test() {
        let db = test_harness::setup_db("users_find_by_email").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        let found = Users::find_by_email("example@example.com".to_string(), &db)
            .await
            .unwrap();
        assert!(found.is_some());
        let user = found.unwrap();
        assert_eq!(
            user,
            Users {
                id: Some(1),
                email: "example@example.com".to_string(),
                has_admin: true,
                has_set_score: true
            }
        );
    }

    #[tokio::test]
    async fn get_or_create_create_test() {
        let db = test_harness::setup_db("users_get_or_create_create").await;
        let req = Users::get_or_create("example@example.com".to_string(), &db).await;
        assert!(req.is_ok());
        assert_eq!(
            req.unwrap(),
            Users {
                id: Some(1),
                email: "example@example.com".to_string(),
                has_admin: false,
                has_set_score: false,
            },
        )
    }

    #[tokio::test]
    async fn get_or_create_get_test() {
        let db = test_harness::setup_db("users_get_or_create_get").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        let req = Users::get_or_create("example@example.com".to_string(), &db).await;
        assert!(req.is_ok());
        assert_eq!(
            req.unwrap(),
            Users {
                id: Some(1),
                email: "example@example.com".to_string(),
                has_admin: true,
                has_set_score: true,
            },
        )
    }

    #[tokio::test]
    async fn insert_test() {
        let db = test_harness::setup_db("users_insert").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn all_test() {
        let db = test_harness::setup_db("users_all").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        assert!(Users::new("example1@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        assert!(Users::new("example2@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        assert!(Users::new("example3@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        assert_eq!(Users::all(&db).await.unwrap().len(), 4);
    }

    #[tokio::test]
    async fn find_by_id_test() {
        let db = test_harness::setup_db("users_find_by_id").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        let found = Users::find_by_id(1, &db).await.unwrap();
        assert!(found.is_some());
        let user = found.unwrap();
        assert_eq!(
            user,
            Users {
                id: Some(1),
                email: "example@example.com".to_string(),
                has_admin: true,
                has_set_score: true
            }
        );
    }

    #[tokio::test]
    async fn update_test() {
        let db = test_harness::setup_db("users_update").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());

        assert!(
            Users::update(&db, 1, "example@example.com".to_string(), true, false)
                .await
                .is_ok()
        );
        assert_eq!(
            Users::find_by_id(1, &db)
                .await
                .unwrap()
                .unwrap()
                .has_set_score,
            false
        );
    }
}
