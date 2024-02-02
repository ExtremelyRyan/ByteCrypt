use crypt_ui::cli::load_cli;
fn main() -> anyhow::Result<()> {
    load_cli();

    let s = String::from("hello");
    let s2: &str = &s;

    Ok(())
}
