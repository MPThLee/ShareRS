use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ids::*, DatabaseError};

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: FileId,
    pub name: String,
    pub original_name: Option<String>,
    pub mime: Option<String>,
    pub views: i64,
    pub max_views: Option<i64>,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDbRequest {
    pub name: String,
    pub original_name: Option<String>,
    pub mime: Option<String>,
    pub max_views: Option<i64>,
    pub user_id: UserId,
}

impl File {
    pub async fn insert(
        file: FileDbRequest,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO files (
                name, original_name, mime, max_views, user_id
            )
            VALUES (
                $1, $2, $3, $4, $5
            )
            ",
            file.name,
            file.original_name,
            file.mime,
            file.max_views,
            file.user_id.0
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get_by_name<'a, E, S>(name: S, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
        S: Into<String>,
    {
        let result = sqlx::query!(
            "
            SELECT 
                f.id, f.name, f.original_name, f.mime, f.views,
                f.max_views, f.user_id, f.created_at
            FROM 
                files f
            WHERE 
                    f.name = $1
                AND (
                    f.max_views IS NULL
                OR
                    f.views < COALESCE(f.max_views, '9223372036854775807'::bigint)
                )
            ",
            &name.into()
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(File {
                id: FileId(row.id),
                name: row.name,
                original_name: row.original_name,
                mime: row.mime,
                views: row.views,
                max_views: row.max_views,
                user_id: UserId(row.user_id),
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

        let files = sqlx::query!(
            "
            SELECT 
                f.id, f.name, f.original_name, f.mime, f.views,
                f.max_views, f.user_id, f.created_at
            FROM 
                files f
            WHERE 
                f.user_id = $1
            ",
            &user_id.0
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|f| File {
                id: FileId(f.id),
                name: f.name,
                original_name: f.original_name,
                mime: f.mime,
                views: f.views,
                max_views: f.max_views,
                user_id: UserId(f.user_id),
                created_at: f.created_at,
            }))
        })
        .try_collect::<Vec<Self>>()
        .await?;

        Ok(files)
    }

    pub async fn get<'a, 'b, E>(id: FileId, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_many(&[id], executor)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(file_ids: &[FileId], exec: E) -> Result<Vec<Self>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let file_ids_parsed: Vec<Uuid> = file_ids.iter().map(|x| x.0).collect();
        let files = sqlx::query!(
            "
            SELECT 
                f.id, f.name, f.original_name, f.mime, f.views,
                f.max_views, f.user_id, f.created_at
            FROM 
                files f
            WHERE 
                f.id = ANY($1)
            ",
            &file_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|f| File {
                id: FileId(f.id),
                name: f.name,
                original_name: f.original_name,
                mime: f.mime,
                views: f.views,
                max_views: f.max_views,
                user_id: UserId(f.user_id),
                created_at: f.created_at,
            }))
        })
        .try_collect::<Vec<Self>>()
        .await?;

        Ok(files)
    }
}
