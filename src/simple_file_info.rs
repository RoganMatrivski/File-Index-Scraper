#[derive(Debug)]
pub struct SimpleFileInfo {
    dir: String,
    file: String,
}

impl SimpleFileInfo {
    #[tracing::instrument(skip(self))]
    pub fn get_full_path(&self) -> String {
        format!("{}{}", self.dir, self.file)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_url_relative_path(&self) -> String {
        self.get_full_path()[2..].to_string()
    }

    #[tracing::instrument(skip(self))]
    pub fn get_decoded_full_path(&self) -> String {
        urlencoding::decode(&self.get_full_path())
            .unwrap()
            .into_owned()
    }

    pub fn new(_dir: String, _file: String) -> SimpleFileInfo {
        SimpleFileInfo {
            dir: _dir,
            file: _file,
        }
    }
}
