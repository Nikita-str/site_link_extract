use site_link_extract::Links;
use site_link_extract::link_unificator::StdUnificator as Uni;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] CLI params 
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short)]
    /// file to save extracted links (if not specified then they will be printed in console)  
    save_to: Option<String>,
    #[clap(short)]
    /// max number of async page loads at the same time (by default: inf)
    max_async: Option<usize>,
    #[clap(short)]
    /// disable printing `+` after each loaded page
    disable_printing: bool,
    #[command(subcommand)]
    cmd: CliCmd,
}

#[derive(Debug, Subcommand)]
enum CliCmd {
    /// load initial links from specified file
    LinksFile{path: String},
    /// specify initial links in console
    Links{links: Vec<String>},
}

impl Cli {
    async fn execute(self) -> anyhow::Result<()> {
        let cli = self;

        let mut links = match &cli.cmd {
            CliCmd::Links { links } => Links::<Uni>::new(links)?,
            CliCmd::LinksFile { path } => Links::<Uni>::new_from_file(path)?,
        };
        println!("initialized with {} links", links.len());
    
        let print = !cli.disable_printing;
        links.extract_all_unique(cli.max_async, print).await?;
    
        match &cli.save_to {
            Some(path) => links.save_to_file(path)?,
            None => println!("{links}"),
        }
        println!("extracted {} links", links.len());
    
        Ok(())
    }
}

// [-] CLI params 
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Cli::parse().execute().await
}


#[tokio::test]
async fn example() {
    let cli = Cli {
        disable_printing: false,
        
        // `Some(1)`  => load link by link (slow)
        // `Some(8)`  => load scoped but no more 8 at the same time
        // `None` [*] => load scoped and without restrictions
        max_async: None,

        cmd: CliCmd::LinksFile { path: "./example/init_links.txt".into() },
        save_to: Some("./example/result.txt".into()),        
    };

    cli.execute().await.expect("execute error")
}
