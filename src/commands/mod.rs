use std::io::{ Error, ErrorKind };

mod echo;
mod pwd;
mod cd;
mod mkdir;
mod cat;
mod cp;
mod mv;
mod rm;
mod ls;

pub fn exec(cmd: &str, args: Vec<String>) -> Result<(), Error> {
    match cmd {
        "echo" => echo::run(args),

        "pwd" => pwd::run()?,

        "cd" => cd::run(args)?,

        "mkdir" => mkdir::run(args)?,

        "cat" => cat::run(args)?,

        "cp" => cp::run(args)?,

        "mv" => mv::run(args)?,

        "rm" => rm::run(args)?,

        "ls" => ls::run(args)?,

        _ => {
            return Err(Error::new(ErrorKind::InvalidInput, format!("command not found: {}", cmd)));
        }
    }
    Ok(())
}
