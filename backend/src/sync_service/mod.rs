use diesel::insert_into;
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Object;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use thiserror::Error;

use crate::db::models;
use crate::db::schema::items::dsl::items;
use crate::firebase_listener::{FirebaseListener, FirebaseListenerErr};

#[derive(Error, Debug)]
pub enum SyncServiceError {
    #[error("Connection error: {0}")]
    ConnectError(String),

    #[error(transparent)]
    FirebaseErr(#[from] FirebaseListenerErr),
}

pub struct SyncService {
    /// Pool for Postgres DB backing up HN data
    db_pool: Pool<diesel_async::AsyncPgConnection>,
    firebase_url: String,
    num_workers: usize,
}
impl SyncService {
    pub fn new(
        firebase_url: String,
        db_pool: Pool<diesel_async::AsyncPgConnection>,
        num_workers: usize,
    ) -> Self {
        Self {
            db_pool,
            num_workers,
            firebase_url,
        }
    }

    pub async fn worker(
        &self,
        min_id: i64,
        max_id: i64,
        mut conn: Object<AsyncPgConnection>,
    ) -> Result<(), SyncServiceError> {
        let fb = FirebaseListener::new(&self.firebase_url)
            .map_err(|_| SyncServiceError::ConnectError("HALP".into()))?;
        for i in min_id..max_id {
            let raw_item = fb.get_item(i).await?;
            insert_into(items)
                .values(Into::<models::Item>::into(raw_item))
                .execute(&mut conn)
                .await
                .expect("FRICK");
        }
        Ok(())
    }

    pub async fn fetch_all_data(&self) -> Result<(), SyncServiceError> {
        Ok(())
    }
}
