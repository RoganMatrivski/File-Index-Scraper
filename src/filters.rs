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
        if self.regex.is_empty() {
            return true;
        }

        // If there's any non-match, return false.
        !self.regex.clone().iter().any(|x| !x.is_match(str))
    }

    pub fn match_all_glob(&self, str: &str) -> bool {
        if self.glob.is_empty() {
            return true;
        }

        !self.glob.clone().iter().any(|x| !x.is_match(str))
    }
}
