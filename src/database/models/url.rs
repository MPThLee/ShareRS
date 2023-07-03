use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ids::*, DatabaseError};

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    pub id: UrlId,
    pub name: String,
    pub destination: String,
    pub views: i64,
    pub max_views: Option<i64>,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlRequest {
    pub name: String,
    pub destination: String,
    pub max_views: Option<i64>,
    pub user_id: UserId,
}

#[allow(dead_code)]
impl Url {
    pub async fn insert(
        url: UrlRequest,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<UrlId, DatabaseError> {
        let ret = sqlx::query!(
            "
            INSERT INTO url (
                name, destination, max_views, user_id
            )
            VALUES (
                $1, $2, $3, $4
            )
            RETURNING
                id
            ",
            url.name,
            url.destination,
            url.max_views,
            url.user_id.0
        )
        .fetch_one(&mut *transaction)
        .await?;

        Ok(UrlId(ret.id))
    }

    pub async fn get_by_name<'a, E, S>(name: S, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
        S: Into<String>,
    {
        let result = sqlx::query!(
            "
            SELECT 
                u.id, u.name, u.destination, u.views,
                u.max_views, u.user_id, u.created_at
            FROM 
                url u
            WHERE 
                    u.name = $1
                AND (
                        u.max_views IS NULL
                    OR
                        u.views < COALESCE(u.max_views, '9223372036854775807'::bigint)
                )
            ",
            &name.into()
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(Url {
                id: UrlId(row.id),
                name: row.name,
                destination: row.destination,
                views: row.views,
                max_views: row.max_views,
                user_id: UserId(row.user_id),
                created_at: row.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn increase_views<'a, E>(url_id: UrlId, executor: E) -> Result<(), DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        sqlx::query!(
            "
            UPDATE url
            SET
                views = views + 1
            WHERE
                id = $1
            ",
            &url_id.0
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn get_many_by_user_id<'a, E>(
        user_id: UserId,
        exec: E,
    ) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let urls = sqlx::query!(
            "
            SELECT 
                u.id, u.name, u.destination, u.views,
                u.max_views, u.user_id, u.created_at
            FROM 
                url u
            WHERE 
                u.user_id = $1
            ",
            &user_id.0
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|u| Url {
                id: UrlId(u.id),
                name: u.name,
                destination: u.destination,
                views: u.views,
                max_views: u.max_views,
                user_id: UserId(u.user_id),
                created_at: u.created_at,
            }))
        })
        .try_collect::<Vec<Self>>()
        .await?;

        Ok(urls)
    }

    pub async fn get_count<'a, 'b, E>(executor: E) -> Result<i64, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let ret = sqlx::query!(
            "
            SELECT 
                COUNT(*) AS count
            FROM url
            " // "
              // SELECT reltuples::bigint AS count
              // FROM pg_class
              // WHERE oid = 'public.url'::regclass;
              // "
        )
        .fetch_one(executor)
        .await?;

        ret.count.ok_or(DatabaseError::Other(
            "Unknown error while get count".to_string(),
        ))
    }

    pub async fn get<'a, 'b, E>(id: UrlId, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_many(&[id], executor)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(url_ids: &[UrlId], exec: E) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let url_ids_parsed: Vec<Uuid> = url_ids.iter().map(|x| x.0).collect();
        let urls = sqlx::query!(
            "
            SELECT 
                u.id, u.name, u.destination, u.views,
                u.max_views, u.user_id, u.created_at
            FROM 
                url u
            WHERE 
                u.id = ANY($1)
            ",
            &url_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|u| Url {
                id: UrlId(u.id),
                name: u.name,
                destination: u.destination,
                views: u.views,
                max_views: u.max_views,
                user_id: UserId(u.user_id),
                created_at: u.created_at,
            }))
        })
        .try_collect::<Vec<Self>>()
        .await?;

        Ok(urls)
    }
}
