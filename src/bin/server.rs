extern crate log;
#[macro_use]
extern crate slog;
extern crate slog_stdlog;
extern crate slog_term;
extern crate packagesbot;

use std::env;
use slog::*;

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
    let logger = Logger::new_root(o!());
    logger.set_drain(slog_term::async_stderr());

    slog_stdlog::set_logger_level(logger, log::LogLevelFilter::Debug).unwrap();

    let conf = EnvConfig::load();

    packagesbot::main(&conf.telegram_token, &conf.botanio_token);
}
