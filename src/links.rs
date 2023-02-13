use futures::{StreamExt, stream::FuturesUnordered, Future};
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

    fn lock_new(&self) -> anyhow::Result<std::sync::MutexGuard<Vec<Url>>> {
        self.new.lock().map_err(|_|anyhow::anyhow!("`Mutex` was poisoned"))
    }

    fn lock_all(&self) -> anyhow::Result<std::sync::MutexGuard<HashSet<Url>>> {
        self.all.lock().map_err(|_|anyhow::anyhow!("`Mutex` was poisoned"))
    }

    fn take_unchecked_url(&mut self) -> anyhow::Result<Option<Url>> {
        let mut new = self.lock_new()?;
        Ok(new.pop())
    }

    pub fn len(&self) -> usize {
        match self.all.lock() {
            Ok(all) => all.len(),
            _ => 0, // not sure
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // [+] help fns for `extract_one_more`
    async fn load(link: impl reqwest::IntoUrl) -> anyhow::Result<String> {
        let body = reqwest::get(link).await?.text().await?;
        Ok(body)
    }

    fn absolute_link(potential_parent: &Url, link: &str) -> anyhow::Result<Url> {
        let abs_link = if link.contains("://") {
            reqwest::Url::from_str(link)?
        } else {
            potential_parent.join(link)?
        };
        Ok(abs_link)
    }
    // [-] help fns for `extract_one_more`
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// # return
    /// * `Ok(true)` when another unchecked url succesfully was checked 
    /// * `Ok(false)` when there no unchecked url
    /// * `Err(..)` when an error occurs
    async fn extract_one_more(mut self) -> anyhow::Result<bool> {
        let Some(link) = self.take_unchecked_url()? else { return Ok(false) };
        let body = Links::load(link.as_str()).await?;

        let body = Document::from(body.as_str());
        let body_links = body
            .find(Name("a"))
            .filter_map(|node| node.attr("href"));

        { // to drop MutexGuard
            let mut new = self.lock_new()?;
            let mut all = self.lock_all()?;

            for potential_new_link in body_links {
                let potential_new_link = Links::absolute_link(&link, potential_new_link)?;

                if !all.contains(&potential_new_link) {
                    let new_link = potential_new_link;
                    all.insert(new_link.clone());
                    new.push(new_link);
                }
            }
        }
      
        Ok(true)
    }

    pub async fn extract_all_unique(&mut self, max_together: Option<usize>, print: bool) -> anyhow::Result<()> {
        let mut awaiter = futures::stream::FuturesUnordered::new();
        let mut links_counter = LinksCounter::new(self, max_together);
        if print { links_counter.print_on() }

        loop {
            links_counter.awaiter_filling(|link|awaiter.push(link.extract_one_more()));
            
            // `if awaiter.is_empty() {..}` is the same case as
            //             `awaiter.next().await` return `None`
            //             this case catched in `awaiter_next_await`

            let smth_awaited = links_counter.awaiter_next_await(&mut awaiter).await?;
            if !smth_awaited { return Ok(()) }
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


struct LinksCounter<'x> {
    links: &'x Links,
    cur_run: usize,
    total_run: usize,
    max_together: Option<usize>,
    print: bool,
}

impl<'x> LinksCounter<'x> {
    pub fn new(links: &'x Links, max_together: Option<usize>) -> Self {
        let max_together = max_together.map(|x|usize::max(1, x));

        Self {
            links,
            cur_run: 0,
            total_run: 0,
            max_together,
            print: false,
        }
    }

    fn print_on(&mut self) {
        self.print = true;
    }

    fn print_another_one(&self) -> anyhow::Result<()> {
        if self.print {
            print!("+");
            std::io::stdout().flush()?;
        }
        Ok(())
    }

    fn print_last(&self) {
        if self.print {
            println!()
        }
    }

    fn awaiter_filling(&mut self, mut awaiter_push: impl FnMut(Links)) {
        'awaiter_filling: loop {
            let can_add = self.max_together.map(|max|self.cur_run < max).unwrap_or(true);
            if !can_add { break 'awaiter_filling }
            
            if self.total_run < self.links.len() {
                self.cur_run += 1;
                self.total_run += 1;
                awaiter_push(self.links.clone()); // awaiter.push(self.links.clone().extract_one_more());
            } else {
                break 'awaiter_filling
            }
        }
    }

    /// # return
    /// * `Ok(true)` => next() future was sucessfully ended 
    /// * `Ok(false)` => there was nothing to await
    async fn awaiter_next_await(&mut self, awaiter: &mut FuturesUnordered<impl Future<Output = anyhow::Result<bool>>>) -> anyhow::Result<bool> {
        match awaiter.next().await {
            Some(result) => {
                self.cur_run -= 1;
                let _ = result?;
                self.print_another_one()?;
                Ok(true)
            }
            None => {
                self.print_last();
                Ok(false)
            }
        }
    }
}
