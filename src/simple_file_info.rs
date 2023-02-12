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

    #[deprecated(
        since = "0.1.4",
        note = "Don't use this. This function assumes the url starts with './' which is incorrect."
    )]
    #[tracing::instrument(skip(self))]
    pub fn get_url_relative_path(&self) -> String {
        self.get_full_path()[2..].to_string()
    }

    #[tracing::instrument(skip(self))]
    pub fn get_decoded_filename(&self) -> String {
        let conv_binary = urlencoding::decode_binary(self.file.as_bytes());

        strbyte_to_utf8(conv_binary.to_vec())
    }

    #[tracing::instrument(skip(self), fields(dir = self.dir, file = self.file))]
    pub fn get_decoded_full_path(&self) -> String {
        let full_path = self.get_full_path();
        let conv_binary = urlencoding::decode_binary(full_path.as_bytes());

        let dec = match std::str::from_utf8(&conv_binary) {
            Ok(str) => str.to_owned(),
            Err(_) => {
                tracing::warn!(
                    path = full_path,
                    "Path is not a valid UTF-8 characters. Will attempt to loosely convert path."
                );

                String::from_utf8_lossy(&conv_binary).into_owned()
            }
        };

        dec
    }

    pub fn new(_dir: String, _file: String) -> SimpleFileInfo {
        SimpleFileInfo {
            dir: _dir,
            file: _file,
        }
    }
}

fn strbyte_to_utf8(str: Vec<u8>) -> String {
    match String::from_utf8(str.clone()) {
        Ok(str) => str,
        Err(_) => {
            let vec_u8_str = str
                .clone()
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            tracing::warn!(
                string = vec_u8_str,
                "String is not a valid UTF-8 characters. Will attempt to loosely convert str."
            );

            String::from_utf8_lossy(&str).into_owned()
        }
    }
}
