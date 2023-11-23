use dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    foo: u16,
    bar: bool,
    baz: String,
    boom: Option<u64>,
}

pub fn load_config() {
    dotenv::dotenv().ok();
    match envy::from_env::<Config>() {
        Ok(config) => println!("{:#?}", config),
        Err(error) => eprintln!("{:#?}", error),
    };
}
