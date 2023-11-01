#[derive(Debug)]
pub struct Filters {
    pub regex: Vec<regex::Regex>,
    pub glob: Vec<globset::GlobMatcher>,
    pub exclude_regex: Vec<regex::Regex>,
    pub exclude_glob: Vec<globset::GlobMatcher>,
}

impl Filters {
    pub fn new(
        regex: Vec<regex::Regex>,
        glob: Vec<globset::GlobMatcher>,
        exclude_regex: Vec<regex::Regex>,
        exclude_glob: Vec<globset::GlobMatcher>,
    ) -> Filters {
        Filters {
            regex,
            glob,
            exclude_regex,
            exclude_glob,
        }
    }

    pub fn match_all_regex(&self, str: &str) -> bool {
        self.regex.iter().all(|x| x.is_match(str))
    }

    pub fn match_all_glob(&self, str: &str) -> bool {
        self.glob.iter().all(|x| x.is_match(str))
    }

    pub fn match_all_regex_exclude(&self, str: &str) -> bool {
        self.exclude_regex.iter().all(|x| !x.is_match(str))
    }

    pub fn match_all_glob_exclude(&self, str: &str) -> bool {
        self.exclude_glob.iter().all(|x| !x.is_match(str))
    }
}
