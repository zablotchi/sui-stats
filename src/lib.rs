use std::sync::Arc;

use anyhow::Context;
use diesel::prelude::Insertable;
use diesel_async::RunQueryDsl;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use schema::sizes;
use sui_indexer_alt_framework::{
    FieldCount, Result, db,
    pipeline::{Processor, concurrent::Handler},
    types::full_checkpoint_content::CheckpointData,
};

mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Insertable, FieldCount)]
#[diesel(table_name = sizes)]
pub struct StoredSize {
    cp_sequence_number: i64,
    cp_summary_bytes: i64,
    cp_signatures_bytes: i64,
    cp_contents_bytes: i64,
    tx_count: i64,
    tx_bytes: i64,
    fx_bytes: i64,
    ev_bytes: i64,
    obj_count: i64,
    obj_bytes: i64,
}

pub struct Sizes;

impl Processor for Sizes {
    const NAME: &'static str = "sizes";

    type Value = StoredSize;

    /// The processing logic for turning a checkpoint into rows of the table.
    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let cp_sequence_number = checkpoint.checkpoint_summary.sequence_number as i64;

        let cp_summary_bytes = bcs::to_bytes(&checkpoint.checkpoint_summary.data())
            .context("Failed to serialize checkpoint summary")?
            .len() as i64;

        let cp_signatures_bytes = bcs::to_bytes(&checkpoint.checkpoint_summary.auth_sig())
            .context("Failed to serialize checkpoint signatures")?
            .len() as i64;

        let cp_contents_bytes = bcs::to_bytes(&checkpoint.checkpoint_contents)
            .context("Failed to serialize checkpoint contents")?
            .len() as i64;

        let tx_count = checkpoint.transactions.len() as i64;

        let mut tx_bytes = 0;
        let mut fx_bytes = 0;
        let mut ev_bytes = 0;
        let mut obj_count = 0;
        let mut obj_bytes = 0;

        for transaction in &checkpoint.transactions {
            tx_bytes += bcs::to_bytes(transaction)
                .context("Failed to serialize transaction")?
                .len() as i64;

            fx_bytes += bcs::to_bytes(&transaction.effects)
                .context("Failed to serialize write set")?
                .len() as i64;

            ev_bytes += bcs::to_bytes(&transaction.events)
                .context("Failed to serialize events")?
                .len() as i64;

            obj_count += transaction.output_objects.len() as i64;

            for object in &transaction.output_objects {
                obj_bytes += bcs::to_bytes(object)
                    .context("Failed to serialize object")?
                    .len() as i64;
            }
        }

        Ok(vec![StoredSize {
            cp_sequence_number,

            cp_summary_bytes,
            cp_signatures_bytes,
            cp_contents_bytes,

            tx_count,
            tx_bytes,
            fx_bytes,
            ev_bytes,

            obj_count,
            obj_bytes,
        }])
    }
}

#[async_trait::async_trait]
impl Handler for Sizes {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        Ok(diesel::insert_into(sizes::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?)
    }
}
