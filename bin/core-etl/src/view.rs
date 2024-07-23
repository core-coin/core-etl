use clap::{Parser, Subcommand};
use config::Config;
use storage::Storage;
#[derive(Parser, Debug)]
pub struct ViewArgs {
    #[command(subcommand)]
    sub: ViewSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ViewSubcommands {
    Block {
        #[clap(flatten)]
        group: BlockGroup,
    },
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct BlockGroup {
    #[clap(short = 's', long, env)]
    number: Option<i64>,
    #[clap(short = 'n', long, env)]
    hash: Option<String>,
}

impl ViewArgs {
    pub async fn exec(
        &self,
        _config: Config,
        storage: Box<dyn Storage + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &self.sub {
            ViewSubcommands::Block {
                group: BlockGroup { number, hash },
            } => {
                let block = if let Some(block_number) = number {
                    storage.get_block_by_number(*block_number).await?
                } else {
                    storage.get_block_by_hash(hash.clone().unwrap()).await?
                };
                println!("Requested block:\n {:#?}", block);
                Ok(())
            }
        }
    }
}
