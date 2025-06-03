use clap::Parser;
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

    indexer
        .concurrent_pipeline(Sizes, ConcurrentConfig::default())
        .await?;

    indexer
        .concurrent_pipeline(TransactionSizes, ConcurrentConfig::default())
        .await?;

    indexer
        .concurrent_pipeline(ObjectSizes, ConcurrentConfig::default())
        .await?;

    indexer
        .concurrent_pipeline(EffectSizes, ConcurrentConfig::default())
        .await?;

    let _ = indexer.run().await?.await;
    Ok(())
}
