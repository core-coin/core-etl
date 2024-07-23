use clap::Parser;
use config::Config;
use provider::Provider;
use storage::Storage;
#[derive(Parser, Debug)]
pub struct ExportArgs {}

impl ExportArgs {
    pub async fn exec(
        &self,
        config: Config,
        provider: Provider,
        storage: Box<dyn Storage + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut worker = etl::ETLWorker::new(config, storage, provider);
        worker.run().await
    }
}
