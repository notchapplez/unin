use std::path::PathBuf;

pub fn confihgure(path: PathBuf, noinstall: bool) {
    //fih
    let (tx, rx) = unin_bin::comms::unin_channel(); //channel
    let mut args = vec!["--prefix=/usr/local"];
}
