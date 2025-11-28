use async_sqlite::rusqlite::{Error as RusqliteError, OptionalExtension};
use async_sqlite::{rusqlite::Row, Pool};

use crate::ternary;

#[derive(Clone, PartialEq, Debug)]
pub struct UserSessions {
    pub id: String,
    pub user_id: i64,
    pub has_admin: bool,
    pub has_set_score: bool,
}

impl UserSessions {
    pub fn new(user_id: i64, has_admin: bool, has_set_score: bool) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            user_id,
            has_admin,
            has_set_score,
        }
    }
    fn map_from_row(row: &Row) -> Result<Self, RusqliteError> {
        Ok(Self {
            id: row.get(0)?,
            user_id: row.get(1)?,
            has_admin: ternary!(row.get(2)? => true, false),
            has_set_score: ternary!(row.get(3)? => true, false),
        })
    }

    pub async fn insert(self, pool: &Pool) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            conn.execute("INSERT INTO user_sessions(id, user_id, has_admin, has_set_score) VALUES (?1, ?2, ?3, ?4);", [self.id, self.user_id.to_string(), ternary!(self.has_admin => 1, 0).to_string(), ternary!(self.has_set_score => 1, 0).to_string()])
                .unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn verify(
        pool: &Pool,
        cookie_session: String,
    ) -> Result<VerifiedSession, async_sqlite::Error> {
        pool.conn(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM user_sessions WHERE id = ?1")?;
            let session = stmt
                .query_one([cookie_session.clone()], |row| Self::map_from_row(row))
                .optional()?;
            match session {
                Some(session) => {
                    log::debug!("DB Session ID: {} (cookie: {cookie_session})", session.id);
                    return Ok(VerifiedSession {
                        id: cookie_session,
                        verified: true,
                        has_admin: session.has_admin,
                        has_set_score: session.has_set_score,
                    });
                }
                None => {
                    log::debug!("No Session found in db");
                    return Ok(VerifiedSession {
                        id: cookie_session,
                        verified: false,
                        has_admin: false,
                        has_set_score: false,
                    });
                }
            }
        })
        .await
    }
}

pub struct VerifiedSession {
    pub id: String,
    pub verified: bool,
    pub has_admin: bool,
    pub has_set_score: bool,
}

#[cfg(test)]
mod tests {
    use crate::{db::users::Users, test_harness};

    use super::*;

    #[tokio::test]
    async fn insert_test() {
        let db = test_harness::setup_db("user_sessions_insert").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        assert!(UserSessions::new(1, true, true).insert(&db).await.is_ok());
    }

    #[tokio::test]
    async fn verify_true_test() {
        let db = test_harness::setup_db("user_sessions_verify_true").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        let session = UserSessions::new(1, true, true);
        assert!(session.clone().insert(&db).await.is_ok());
        let verified_session = UserSessions::verify(&db, session.id.clone()).await;
        assert!(verified_session.is_ok());
        let verified = verified_session.unwrap();
        assert_eq!(verified.verified, true);
        assert_eq!(verified.id, session.id);
    }

    #[tokio::test]
    async fn verify_false_test() {
        let db = test_harness::setup_db("user_sessions_verify_false").await;
        assert!(Users::new("example@example.com".to_string(), true, true)
            .insert(&db)
            .await
            .is_ok());
        let verified_session = UserSessions::verify(&db, "HelloWorld".to_string()).await;
        assert!(verified_session.is_ok());
        let verified = verified_session.unwrap();
        assert_eq!(verified.verified, false);
    }
}
