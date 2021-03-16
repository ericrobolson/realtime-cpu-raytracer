pub fn load(file: &'static str) -> Vec<u8> {
    /*
    // Debugging for file paths
        let paths = std::fs::read_dir("./").unwrap();

        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }

    */

    let data = std::fs::read(&file).unwrap();
    data
}
