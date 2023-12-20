use crypt_core::common::get_crypt_folder;
use lazy_static::lazy_static;

lazy_static! {
    ///Path for the google user token
    pub static ref GOOGLE_TOKEN_PATH: String = {
        let mut path = get_crypt_folder();
        path.push(".config");

        if !path.exists() {
            _ = std::fs::create_dir(&path);
        }
        path.push(".google");
        format!("{}", path.display())
    };

    ///Path for the dropbox user token
    pub static ref DROPBOX_TOKEN_PATH: String = {
        let mut path = get_crypt_folder();
        path.push(".config");

        if !path.exists() {
            _ = std::fs::create_dir(&path);
        }
        path.push(".dropbox");
        format!("{}", path.display())
    };
}
