use reqwest::Url;


pub trait LinkUnificator {
    type Unified;
    fn unify(link: &Url) -> Self::Unified;
}


pub struct EmptyUnificator;
impl LinkUnificator for EmptyUnificator {
    type Unified = String;
    fn unify(link: &Url) -> Self::Unified {
        link.to_string()
    }
}


pub struct StdUnificator;
impl LinkUnificator for StdUnificator {
    type Unified = StdUnified;
    fn unify(link: &Url) -> Self::Unified {
        let link = link.as_str();
        let link_wo_prefix = link.split_once("://").map(|(_, right)|right).unwrap_or(&link);
        let unified = link_wo_prefix.split_once("#").map(|(left, _)|left).unwrap_or(link_wo_prefix);
        Self::Unified { unified: unified.into() }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct StdUnified {
    unified: String
}
impl std::fmt::Display for StdUnified {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://{}", self.unified)
    }
}
