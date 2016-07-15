#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hyper;
extern crate rustc_serialize;
extern crate telegram_bot;
extern crate marksman_escape;
#[macro_use]
extern crate quick_error;
extern crate url;

mod botanio;
mod crates;
mod error;
mod telegram;

pub fn main(telegram_token: &str, botanio_token: &str) {
    let bot = telegram::CratesIoBot::new(telegram_token, botanio_token).unwrap();
    bot.run();
}
