use std::{env, fs::File, io, path::PathBuf, process::Command};

const DOWNLOAD_LINK: &str = "https://www.fontsquirrel.com/fonts/download/noto-serif";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let font_zip_path = out_dir.join("noto-serif.zip");
    let font_dir_path = out_dir.join("noto-serif");

    if !font_zip_path.exists() {
        let mut download = ureq::get(DOWNLOAD_LINK).call().unwrap().into_reader();
        let mut file = File::create(&font_zip_path).unwrap();
        io::copy(&mut download, &mut file).unwrap();
    }

    if !font_dir_path.exists() {
        Command::new("unzip")
            .arg(&font_zip_path)
            .arg("-d")
            .arg(&font_dir_path)
            .output()
            .unwrap();
    }
}
