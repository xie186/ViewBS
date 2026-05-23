#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "cli")]
fn main() {
    if let Err(error) = cli::run() {
        eprintln!("Error: {error}");
        std::process::exit(cli::exit_code(&error));
    }
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("The ViewBS binary requires the `cli` feature.");
    std::process::exit(2);
}
