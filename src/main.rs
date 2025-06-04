use clap::Parser;
use std::thread;
use sui_indexer_alt_framework::{
    Result,
    cluster::{self, IndexerCluster},
    pipeline::concurrent::ConcurrentConfig,
};
use sui_sizes::{EffectSizes, MIGRATIONS, ObjectSizes, Sizes, TransactionSizes};
use url::Url;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(
        long,
        default_value = "postgres://postgres:postgrespw@localhost:5432/sui_sizes"
    )]
    database_url: Url,

    #[clap(flatten)]
    cluster_args: cluster::Args,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut indexer =
        IndexerCluster::new(args.database_url, args.cluster_args, Some(&MIGRATIONS)).await?;

    // Detect the number of CPU cores and create a custom config
    let num_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(5); // fallback to 5 if detection fails

    println!(
        "Detected {} CPU cores, setting write concurrency to {}",
        num_cores, num_cores
    );

    let concurrent_config = ConcurrentConfig {
        committer: sui_indexer_alt_framework::pipeline::CommitterConfig {
            write_concurrency: num_cores,
            collect_interval_ms: 500,
            watermark_interval_ms: 500,
        },
        pruner: None,
    };

    indexer
        .concurrent_pipeline(Sizes, concurrent_config.clone())
        .await?;

    indexer
        .concurrent_pipeline(TransactionSizes, concurrent_config.clone())
        .await?;

    indexer
        .concurrent_pipeline(ObjectSizes, concurrent_config.clone())
        .await?;

    indexer
        .concurrent_pipeline(EffectSizes, concurrent_config.clone())
        .await?;

    let _ = indexer.run().await?.await;
    Ok(())
}
