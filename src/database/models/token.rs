use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ids::*, DatabaseError};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: TokenId,
    pub user_id: UserId,
    pub expires: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRequest {
    pub user_id: UserId,
    pub expires: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
impl Token {
    pub async fn insert(
        token: TokenRequest,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<TokenId, DatabaseError> {
        let ret = sqlx::query!(
            "
            INSERT INTO token (
                user_id, expires
            )
            VALUES (
                $1, $2
            )
            RETURNING
                id
            ",
            token.user_id.0,
            token.expires
        )
        .fetch_one(&mut *transaction)
        .await?;

        Ok(TokenId(ret.id))
    }

    pub async fn get_valid_by_id<'a, E>(
        token_id: TokenId,
        executor: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT 
                t.id, t.user_id, t.expires, t.created_at
            FROM 
                token t
            WHERE 
                    t.id = $1
                AND 
                    CURRENT_TIMESTAMP < COALESCE(t.expires, 'infinity')
            ",
            &token_id.0
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(Token {
                id: TokenId(row.id),
                user_id: UserId(row.user_id),
                expires: row.expires,
                created_at: row.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_by_user_id<'a, E>(
        user_id: UserId,
        exec: E,
    ) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let tokens = sqlx::query!(
            "
            SELECT 
                t.id, t.user_id, t.expires, t.created_at
            FROM 
                token t
            WHERE 
                t.user_id = $1
            ",
            &user_id.0
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|t| Token {
                id: TokenId(t.id),
                user_id: UserId(t.user_id),
                expires: t.expires,
                created_at: t.created_at,
            }))
        })
        .try_collect::<Vec<Self>>()
        .await?;

        Ok(tokens)
    }

    pub async fn get<'a, 'b, E>(id: TokenId, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_many(&[id], executor)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(token_ids: &[TokenId], exec: E) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let token_ids_parsed: Vec<Uuid> = token_ids.iter().map(|x| x.0).collect();
        let tokens = sqlx::query!(
            "
            SELECT 
                t.id, t.user_id, t.expires, t.created_at
            FROM 
                token t
            WHERE 
                t.id = ANY($1)
            ",
            &token_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|t| Token {
                id: TokenId(t.id),
                user_id: UserId(t.user_id),
                expires: t.expires,
                created_at: t.created_at,
            }))
        })
        .try_collect::<Vec<Self>>()
        .await?;

        Ok(tokens)
    }
}
