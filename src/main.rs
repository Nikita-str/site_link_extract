use site_link_extract::Links;
use site_link_extract::link_unificator::StdUnificator as Uni;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // must be run from repo folder (see relative path)
    let mut links = Links::<Uni>::new_from_file("./example/init_links.txt")?;
    // or use:
    // let mut links = Links::new(vec!["https://www.superbad.com"])?;
    println!("initialized with {} links", links.len());
    
    // // load link by link (slow):  
    // links.extract_all_unique(Some(1), true).await?;

    // // load scoped but no more 8 at the same time:
    // links.extract_all_unique(Some(8), true).await?;
    
    // load scoped and without restrictions:
    links.extract_all_unique(None, true).await?;

    links.save_to_file("./example/result.txt")?;
    println!("extracted {} links", links.len());

    Ok(())
}

