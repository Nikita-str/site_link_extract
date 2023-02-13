use site_link_extract::Links;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut links = Links::new(vec!["https://www.superbad.com"]);
    
    // // load link by link (slow):  
    // links.extract_all_unique(Some(1), true).await?;

    // // load scoped but no more 8 at the same time:
    // links.extract_all_unique(Some(8), true).await?;
    
    // load scoped and without restrictions:
    links.extract_all_unique(None, true).await?;
    
    println!("{links}");
    Ok(())
}

