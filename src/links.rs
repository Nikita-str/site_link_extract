use futures::StreamExt;
use reqwest::Url;
use std::collections::HashSet;
use std::io::Write;
use std::str::FromStr;

use select::document::Document;
use select::predicate::Name;

use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Links {
    all: Arc<Mutex<HashSet<Url>>>,
    new: Arc<Mutex<Vec<Url>>>,
}

impl Links {
    /// # panic
    /// * if `init_links` is empty
    /// * if any links from `init_links` is bad url
    pub fn new<'x>(init_links: impl IntoIterator<Item = &'x str>) -> Self {
        let all = HashSet::from_iter(init_links.into_iter().map(
            |link|Url::from_str(link).unwrap_or_else(|e|panic!("bad URL({e}): {link}"))
        ));
        if all.is_empty() { panic!("`init_links` was empty") }

        let new = Vec::from_iter(all.iter().map(|x|x.clone()));

        Self {
            all: Arc::new(Mutex::new(all)),
            new: Arc::new(Mutex::new(new)),
        }
    }

    /// # return
    /// * `Ok(true)` when there still exist unchecked links
    /// * `Ok(false)` when nothing more to check
    /// * `Err(..)` when an error occurs
    pub async fn next(self) -> anyhow::Result<bool> {
        let link = { // to drop MutexGuard
            let Ok(mut new) = self.new.lock() else { anyhow::bail!("`Mutex` was poisoned") };
            let Some(link) = new.pop() else { return Ok(false) };
            link
        };

        let body = reqwest::get(link.as_str())
            .await?
            .text()
            .await?;

        let body_links = Document::from(body.as_str());
        let body_links = body_links
            .find(Name("a"))
            .filter_map(|node| node.attr("href"));

        { // to drop MutexGuard
            let Ok(mut new) = self.new.lock() else { anyhow::bail!("`Mutex` was poisoned") };
            let Ok(mut all) = self.all.lock() else { anyhow::bail!("`Mutex` was poisoned") };

            for potential_new_link in body_links {
                let url = reqwest::Url::from_str(link.as_str())?;

                let potential_new_link = if potential_new_link.contains("://") {
                    reqwest::Url::from_str(potential_new_link)?
                } else {
                    url.join(potential_new_link)?
                };

                if !all.contains(&potential_new_link) {
                    let new_link = potential_new_link;
                    all.insert(new_link.clone());
                    new.push(new_link);
                }
            }
        }
      
        Ok(true)
    }


    pub async fn take_all_unique(&mut self, max_together: Option<usize>, print: bool) -> anyhow::Result<()> {
        let max_together = max_together.map(|x|usize::max(1, x));

        let mut cur_run = 0;
        let mut total_run = 0;

        let mut awaited = futures::stream::FuturesUnordered::new();

        loop {
            'awaiter_filling: loop {
                let can_add = max_together.map(|max|cur_run < max).unwrap_or(true);
                if !can_add { break 'awaiter_filling }
                
                if total_run < self.len() {
                    cur_run += 1;
                    total_run += 1;
                    awaited.push(self.clone().next());
                } else {
                    break 'awaiter_filling
                }
            }

            // `if awaited.is_empty() {..}` is the same case as
            //             `awaited.next().await` return `None`

            match awaited.next().await {
                Some(result) => {
                    cur_run -= 1;
                    let _ = result?;            
                    if print {
                        print!("+");
                        std::io::stdout().flush()?;
                    }
                }
                None => {
                    if print { println!() }
                    return Ok(())
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        match self.all.lock() {
            Ok(all) => all.len(),
            _ => 0, // not sure
        }
    }
}

impl std::fmt::Display for Links {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}]:", self.len())?;
        let Ok(all) = self.all.lock() else { return Ok(()) };
        for link in all.iter() {
            writeln!(f, "{link}")?
        }
        Ok(())
    }
}