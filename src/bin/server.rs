#[macro_use]
extern crate log;
extern crate env_logger;
extern crate packagesbot;

use std::env;

struct EnvConfig {
    telegram_token: String,
    botanio_token: String,
}

impl EnvConfig {
    fn load() -> EnvConfig {
        let tg_token = env::var("TELEGRAM_TOKEN").unwrap();
        let bt_token = env::var("BOTANIO_TOKEN").unwrap();

        EnvConfig {
            telegram_token: tg_token,
            botanio_token: bt_token,
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    let conf = EnvConfig::load();

    packagesbot::main(&conf.telegram_token, &conf.botanio_token);
}
