#[macro_export]
macro_rules! temp {
    ($file:expr) => {{
        log::trace!("CREATING TMP FILE");
        let mut tempdir = std::env::temp_dir();
        log::trace!("TMP DIR: {:?}", tempdir);
        tempdir.push($file);
        if tempdir.exists() {
            std::fs::remove_file(&tempdir).unwrap();
        }
        tempdir.clone().to_str().unwrap()
    }};
}
