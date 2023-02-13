#[derive(Debug)]
pub struct Filters {
    pub regex: Vec<regex::Regex>,
    pub glob: Vec<globset::GlobMatcher>,
}

impl Filters {
    pub fn new(regex: Vec<regex::Regex>, glob: Vec<globset::GlobMatcher>) -> Filters {
        Filters { regex, glob }
    }

    pub fn match_all_regex(&self, str: &str) -> bool {
        !self.regex.iter().any(|x| !x.is_match(str))
    }

    pub fn match_all_glob(&self, str: &str) -> bool {
        !self.glob.iter().any(|x| !x.is_match(str))
    }
}
