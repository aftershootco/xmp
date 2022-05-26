use std::path::Path;
use xmp::*;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let argc = args.len();

    if argc < 2 {
        eprintln!("Give filename");
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);

    // let u = UpdateResults {
    //     datetime: Some(1690139150),
    //     ..Default::default()
    // };
    // u.update(path)?;

    let r = OptionalResults::load(path)?;
    println!("{:#?}", r);
    Ok(())
}
