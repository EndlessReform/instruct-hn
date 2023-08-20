use diesel::dsl::max;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel_async::pooled_connection::deadpool::Object;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use std::vec;
use thiserror::Error;

use crate::db::models;
use crate::db::schema::items::dsl::*;
use crate::firebase_listener::{FirebaseListener, FirebaseListenerErr};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    ConnectError(String),

    #[error(transparent)]
    FirebaseError(#[from] FirebaseListenerErr),

    #[error(transparent)]
    DieselError(#[from] DieselError),
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
    ) -> Result<(), Error> {
        let fb = FirebaseListener::new(&self.firebase_url)
            .map_err(|_| Error::ConnectError("HALP".into()))?;
        for i in min_id..max_id {
            let raw_item = fb.get_item(i).await?;
            let item = Into::<models::Item>::into(raw_item);
            println!("Uploading {}", i);
            insert_into(items)
                .values(&item)
                .on_conflict(crate::db::schema::items::id)
                .do_update()
                .set(&item)
                .execute(&mut conn)
                .await?;
        }
        Ok(())
    }

    /// `divide_ranges` somewhat fairly distributes the catchup range among workers
    fn divide_ranges(&self, min_id: i64, max_id: i64) -> Vec<(i64, i64)> {
        let coerced_nworkers: i64 = self.num_workers as i64;
        if min_id >= max_id {
            return vec![];
        } else if max_id - min_id <= coerced_nworkers {
            // Potentially can't saturate workers, so give as many as possible an ID
            return (min_id..max_id).map(|i| (i, i)).collect();
        } else {
            let num_ids_per_worker = (max_id - min_id) / coerced_nworkers;

            let mut res: Vec<(i64, i64)> = (0..coerced_nworkers)
                .map(|i| {
                    (
                        min_id + i * num_ids_per_worker,
                        min_id + (i + 1) * num_ids_per_worker - 1,
                    )
                })
                .collect();

            // for convenience, give the remainder to the last: it's only on the order of ~20, which compared to thousands of items is nothing
            if let Some(i) = res.last_mut() {
                *i = (i.0, max_id);
            }

            return res;
        }
    }

    pub async fn fetch_all_data(&self) -> Result<(), Error> {
        let fb = FirebaseListener::new(&self.firebase_url)?;
        let max_fb_id = fb.get_max_id().await?;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|_| Error::ConnectError("Listener could not access db pool!".into()))?;

        let max_db_item: Option<i64> = items.select(max(id)).first(&mut conn).await?;
        let id_ranges = self.divide_ranges(
            max_db_item.ok_or(Error::ConnectError(
                "Cannot find max DB item in Postgres!".into(),
            ))?,
            max_fb_id,
        );
        println!("Draft range: {:?}", id_ranges);
        Ok(())
    }
}
