use std::io::Error;
use std::env;

pub fn run() -> Result<(), Error> {
    println!("{}", env::current_dir()?.display());
    Ok(())
}
