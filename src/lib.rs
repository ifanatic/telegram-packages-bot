#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate marksman_escape;
#[macro_use]
extern crate quick_error;
extern crate regex;
extern crate rustc_serialize;
extern crate select;
extern crate telegram_bot;
extern crate tendril;
extern crate threadpool;
extern crate url;

use std::collections::BTreeMap;

mod crates;
mod error;
mod godoc;
mod telegram;

pub struct Package {
    pub name: String,
    pub repository: Option<String>,
    pub description: Option<String>,
}

impl Package {
    fn empty() -> Package {
        Package {
            name: String::new(),
            repository: None,
            description: None,
        }
    }
}

pub trait SearchRepo: SearchRepoClone + Send {
    fn search(&self, query: &str) -> Result<Vec<Package>, error::Error>;
}

pub trait SearchRepoClone {
    fn clone_box(&self) -> Box<SearchRepo>;
}

impl<T> SearchRepoClone for T
    where T: 'static + SearchRepo + Clone
{
    fn clone_box(&self) -> Box<SearchRepo> {
        Box::new(self.clone())
    }
}

impl Clone for Box<SearchRepo> {
    fn clone(&self) -> Box<SearchRepo> {
        self.clone_box()
    }
}

pub fn main(telegram_token: &str, botanio_token: &str) {
    let repos: Vec<(&str, Box<SearchRepo>)> = vec![("rust", Box::new(crates::CratesRepo {})),
                                                   ("go", Box::new(godoc::GodocRepo {}))];

    let bot = telegram::bot::PkgsBot::new(telegram_token, botanio_token, repos).unwrap();
    bot.run();
}
