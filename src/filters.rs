#[derive(Debug)]
pub struct Filters {
    pub regex: Vec<regex::Regex>,
}

impl Filters {
    pub fn new(regex: Vec<regex::Regex>) -> Filters {
        Filters { regex }
    }

    pub fn match_all_regex(&self, str: &str) -> bool {
        // If there's any non-match, return false.
        !self.regex.clone().into_iter().any(|x| !x.is_match(str))
    }
}
