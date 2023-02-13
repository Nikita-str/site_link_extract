use site_link_extract::Links;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut links = Links::new(vec!["https://www.superbad.com"]);
    links.take_all_unique(true).await?;
    println!("{links}");
    Ok(())
}

