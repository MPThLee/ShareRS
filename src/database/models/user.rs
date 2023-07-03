use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ids::UserId, DatabaseError};

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserId,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAuth {
    pub username: String,
    pub password: String,
}

#[allow(dead_code)]
impl User {
    pub async fn insert(
        user: UserAuth,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let password_hash = crate::modules::password::hash(user.password).await?;
        sqlx::query!(
            "
            INSERT INTO users (
                username, password
            )
            VALUES (
                $1, $2
            )
            ",
            user.username,
            password_hash
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get_by_username<'a, E>(
        username: String,
        executor: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT 
                u.id, u.username, u.password,
                u.is_active, u.is_admin, u.created_at
            FROM 
                users u
            WHERE 
                u.username = $1
            ",
            &username
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(User {
                id: UserId(row.id),
                username: row.username,
                password: row.password,
                is_active: row.is_active,
                is_admin: row.is_admin,
                created_at: row.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get<'a, 'b, E>(id: UserId, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_many(&[id], executor)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(user_ids: &[UserId], exec: E) -> Result<Vec<User>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let user_ids_parsed: Vec<Uuid> = user_ids.iter().map(|x| x.0).collect();
        let users = sqlx::query!(
            "
            SELECT 
                u.id, u.username, u.password,
                u.is_active, u.is_admin, u.created_at
            FROM 
                users u
            WHERE 
                u.id = ANY($1)
            ",
            &user_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|u| User {
                id: UserId(u.id),
                username: u.username,
                password: u.password,
                is_active: u.is_active,
                is_admin: u.is_admin,
                created_at: u.created_at,
            }))
        })
        .try_collect::<Vec<User>>()
        .await?;

        Ok(users)
    }
}
