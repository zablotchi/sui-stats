use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use anyhow::Context;
use diesel::prelude::Insertable;
use diesel_async::RunQueryDsl;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use schema::{effect_sizes, object_sizes, sizes, transaction_sizes};
use sui_indexer_alt_framework::{
    FieldCount, Result, db,
    pipeline::{Processor, concurrent::Handler},
    types::full_checkpoint_content::CheckpointData,
};

mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Global counters for tracking progress
static PROCESSED_COUNT: AtomicU64 = AtomicU64::new(0);
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

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
    unique_object_ids: i64,
    unique_event_ids: i64,
}

#[derive(Insertable, FieldCount)]
#[diesel(table_name = transaction_sizes)]
pub struct StoredTransactionSize {
    tx_digest: String,
    cp_sequence_number: i64,
    tx_size_bytes: i64,
}

#[derive(Insertable, FieldCount)]
#[diesel(table_name = object_sizes)]
pub struct StoredObjectSize {
    object_id: String,
    version: i64,
    cp_sequence_number: i64,
    object_size_bytes: i64,
    is_input: bool,
}

#[derive(Insertable, FieldCount)]
#[diesel(table_name = effect_sizes)]
pub struct StoredEffectSize {
    tx_digest: String,
    cp_sequence_number: i64,
    effect_size_bytes: i64,
}

pub struct Sizes;

impl Processor for Sizes {
    const NAME: &'static str = "sizes";

    type Value = StoredSize;

    /// The processing logic for turning a checkpoint into rows of the table.
    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let cp_sequence_number = checkpoint.checkpoint_summary.sequence_number as i64;

        // Initialize start time on first checkpoint
        let start_time = START_TIME.get_or_init(|| Instant::now());

        // Increment processed count
        let current_count = PROCESSED_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

        // Print progress every 1000 checkpoints
        if current_count % 1000 == 0 {
            let elapsed = start_time.elapsed();
            let rate = current_count as f64 / elapsed.as_secs_f64();
            println!(
                "Progress: Processed {} checkpoints (current: {}) | Rate: {:.2} checkpoints/sec | Elapsed: {:.2}s",
                current_count,
                cp_sequence_number,
                rate,
                elapsed.as_secs_f64()
            );
        }

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

        // Track unique object IDs and event IDs across all transactions
        let mut unique_object_ids = HashSet::new();
        let mut unique_event_ids = HashSet::new();

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

            // Collect unique object IDs from input objects
            for object in &transaction.input_objects {
                unique_object_ids.insert(object.id());
            }

            // Collect unique object IDs from output objects and calculate bytes
            for object in &transaction.output_objects {
                unique_object_ids.insert(object.id());
                obj_bytes += bcs::to_bytes(object)
                    .context("Failed to serialize object")?
                    .len() as i64;
            }

            // Collect unique event IDs from transaction events
            if let Some(events) = &transaction.events {
                for event in &events.data {
                    unique_event_ids.insert(event.package_id);
                }
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
            unique_object_ids: unique_object_ids.len() as i64,
            unique_event_ids: unique_event_ids.len() as i64,
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

pub struct TransactionSizes;

impl Processor for TransactionSizes {
    const NAME: &'static str = "transaction_sizes";

    type Value = StoredTransactionSize;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let cp_sequence_number = checkpoint.checkpoint_summary.sequence_number as i64;
        let mut result = Vec::new();

        for transaction in &checkpoint.transactions {
            let tx_digest = transaction.transaction.digest().to_string();
            let tx_size_bytes = bcs::to_bytes(transaction)
                .context("Failed to serialize transaction")?
                .len() as i64;

            result.push(StoredTransactionSize {
                tx_digest,
                cp_sequence_number,
                tx_size_bytes,
            });
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl Handler for TransactionSizes {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        Ok(diesel::insert_into(transaction_sizes::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?)
    }
}

pub struct ObjectSizes;

impl Processor for ObjectSizes {
    const NAME: &'static str = "object_sizes";

    type Value = StoredObjectSize;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let cp_sequence_number = checkpoint.checkpoint_summary.sequence_number as i64;
        let mut result = Vec::new();

        for transaction in &checkpoint.transactions {
            // Process input objects
            for object in &transaction.input_objects {
                let object_id = object.id().to_string();
                let version = object.version().value() as i64;
                let object_size_bytes = bcs::to_bytes(object)
                    .context("Failed to serialize input object")?
                    .len() as i64;

                result.push(StoredObjectSize {
                    object_id,
                    version,
                    cp_sequence_number,
                    object_size_bytes,
                    is_input: true,
                });
            }

            // Process output objects
            for object in &transaction.output_objects {
                let object_id = object.id().to_string();
                let version = object.version().value() as i64;
                let object_size_bytes = bcs::to_bytes(object)
                    .context("Failed to serialize output object")?
                    .len() as i64;

                result.push(StoredObjectSize {
                    object_id,
                    version,
                    cp_sequence_number,
                    object_size_bytes,
                    is_input: false,
                });
            }
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl Handler for ObjectSizes {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        Ok(diesel::insert_into(object_sizes::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?)
    }
}

pub struct EffectSizes;

impl Processor for EffectSizes {
    const NAME: &'static str = "effect_sizes";

    type Value = StoredEffectSize;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let cp_sequence_number = checkpoint.checkpoint_summary.sequence_number as i64;
        let mut result = Vec::new();

        for transaction in &checkpoint.transactions {
            let tx_digest = transaction.transaction.digest().to_string();
            let effect_size_bytes = bcs::to_bytes(&transaction.effects)
                .context("Failed to serialize effects")?
                .len() as i64;

            result.push(StoredEffectSize {
                tx_digest,
                cp_sequence_number,
                effect_size_bytes,
            });
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl Handler for EffectSizes {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        Ok(diesel::insert_into(effect_sizes::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?)
    }
}
