use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() {
    copy_res().unwrap();
}

/// Copy 'res' folder
pub fn copy_res() -> Result<()> {
    // Copy assets
    println!("cargo:rerun-if-changed=*");

    // Copy the files to the same directory as the executable
    let out_dir = format!("{}/../../../", std::env::var("OUT_DIR").unwrap());

    // Execute the copy, including the res folder
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = vec![];
    paths_to_copy.push("./res/");
    copy_items(&paths_to_copy, out_dir, &copy_options).unwrap();

    Ok(())
}
