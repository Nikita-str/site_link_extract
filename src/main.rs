use reqwest::Url;
use std::collections::HashSet;
use std::io::Write;
use std::str::FromStr;

use select::document::Document;
use select::predicate::Name;

struct Links {
    all: HashSet<Url>,
    new: Vec<Url>,
}

impl Links {
    pub fn new<'x>(init_links: impl IntoIterator<Item = &'x str>) -> Self {
        let all = HashSet::from_iter(init_links.into_iter().map(
            |link|Url::from_str(link).unwrap_or_else(|e|panic!("bad URL({e}): {link}"))
        ));
        if all.is_empty() { panic!("`init_links` was empty") }

        let new = Vec::from_iter(all.iter().map(|x|x.clone()));

        Self {
            all,
            new,
        }
    }

    /// # return
    /// * `Ok(true)` when there still exist unchecked links
    /// * `Ok(false)` when nothing more to check
    /// * `Err(..)` when an error occurs
    pub async fn next(&mut self) -> anyhow::Result<bool> {
        let Some(link) = self.new.pop() else { return Ok(false) };

        let body = reqwest::get(link.as_str())
            .await?
            .text()
            .await?;

        let body_links = Document::from(body.as_str());
        let body_links = body_links
            .find(Name("a"))
            .filter_map(|node| node.attr("href"));

        for potential_new_link in body_links {
            let url = reqwest::Url::from_str(link.as_str())?;

            let potential_new_link = if potential_new_link.contains("://") {
                reqwest::Url::from_str(potential_new_link)?
            } else {
                url.join(potential_new_link)?
            };

            if !self.all.contains(&potential_new_link) {
                let new_link = potential_new_link;
                self.all.insert(new_link.clone());
                self.new.push(new_link);
            }
        }
      
        Ok(true)
    }

    pub async fn take_all_unique(&mut self, print: bool) -> anyhow::Result<()> {
        loop {
            if print { 
                print!("+");
                let _ = std::io::stdout().flush();
            }

            if !self.next().await? {
                if print { println!() }
                return Ok(())
            }
        }
    }

    pub fn len(&self) -> usize {
        self.all.len()
    }
}

impl std::fmt::Display for Links {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}]:", self.len())?;
        for link in &self.all {
            writeln!(f, "{link}")?
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut links = Links::new(vec!["https://www.superbad.com"]);
    links.take_all_unique(true).await?;
    println!("{links}");
    Ok(())
}

