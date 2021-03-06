use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::iter::Iterator;
use std::sync::Arc;
use regex::Regex;
use rustc_serialize::{Encodable, Decodable};
use telegram_bot::{Api, ListeningAction, ListeningMethod, MessageType};
use telegram_bot::types::{Integer, ParseMode};
use threadpool::ThreadPool;
use telegram::botanio::Botan;
use telegram::html::HtmlMessageBuilder;
use super::super::{SearchRepo, Package};
use error::Error;

const MAX_MESSAGE_LENGTH: usize = 4096;
const SEARCH_WORKERS_COUNT: usize = 4;
lazy_static!(
    static ref COMMAND_RE: Regex = Regex::new(r"/(\w+)\s*(.*)").unwrap();
);

trait StateStore {
    fn save<S: Encodable>(&self, uid: i64, cid: i64);
    fn load<M: Decodable>(&self, uid: i64, cid: i64);
}

#[derive(Debug, RustcEncodable)]
struct SearchStatsMessage {
    query: String,
}

#[derive(Clone)]
pub struct BotContext<'a> {
    api: &'a Api,
    tracker: &'a Botan,
}

impl<'a> BotContext<'a> {
    fn new(api: &'a Api, tracker: &'a Botan) -> BotContext<'a> {
        BotContext {
            api: api,
            tracker: tracker,
        }
    }
}

pub struct RequestContext<'a> {
    bot_ctx: BotContext<'a>,
    chat_id: i64,
    uid: i64,
}

impl<'a> RequestContext<'a> {
    fn new(bot_ctx: BotContext<'a>, cid: i64, uid: i64) -> RequestContext<'a> {
        RequestContext {
            bot_ctx: bot_ctx,
            chat_id: cid,
            uid: uid,
        }
    }
}

pub struct Command<'a> {
    name: &'a str,
    query: &'a str,
}

impl<'a> Command<'a> {
    fn parse(t: &'a str) -> Option<Command<'a>> {
        if let Some(caps) = COMMAND_RE.captures(t) {
            let cmd = caps.at(1).unwrap();
            Some(Command {
                name: cmd,
                query: caps.at(2).unwrap().trim(),
            })
        } else {
            None
        }
    }
}

pub trait CommandHandler {
    fn handle(&self, ctx: &RequestContext, cmd: &Command) -> Result<(), Error>;
}

struct SearchHandler {
    repo: Box<SearchRepo>,
    pool: Arc<ThreadPool>,
}

impl SearchHandler {
    fn new(repo: Box<SearchRepo>, pool: Arc<ThreadPool>) -> SearchHandler {
        SearchHandler {
            repo: repo,
            pool: pool,
        }
    }

    fn send_empty_result(api: &Api, chat_id: Integer) -> Result<(), Error> {
        let msg = "No results found";
        try!(api.send_message(chat_id,
                                   msg.to_owned(),
                                   Some(ParseMode::Html),
                                   None,
                                   None,
                                   None));

        Ok(())
    }

    fn send_packages(api: &Api, chat_id: Integer, packages: &[Package]) -> Result<(), Error> {

        let mut msg = String::with_capacity(1024);

        for pkg in packages.iter().take(10) {
            let msg_part = SearchHandler::prepare_message_text(pkg);

            if msg.len() + msg_part.len() >= MAX_MESSAGE_LENGTH {
                break;
            }

            msg.push_str(&msg_part);
        }

        try!(api.send_message(chat_id, msg, Some(ParseMode::Html), Some(true), None, None));

        Ok(())
    }

    fn prepare_message_text(pkg: &Package) -> String {
        let description: &str = match pkg.description {
            Some(ref desc) => desc,
            None => "",
        };
        let repo_url: &str = match pkg.repository {
            Some(ref u) => u,
            None => "",
        };
        let doc_url: &str = match pkg.documentation {
            Some(ref u) => u,
            None => "",
        };
        SearchHandler::render_html_message(&pkg.name, description, repo_url, doc_url)
    }

    fn render_html_message(name: &str, description: &str, url: &str, doc_url: &str) -> String {
        let mut msg_builder = HtmlMessageBuilder::new();
        {
            msg_builder.name(name);
        }

        if !description.is_empty() {
            msg_builder.description(description);
        }

        if !url.is_empty() {
            msg_builder.repo_url(url);
        }

        if !doc_url.is_empty() {
            msg_builder.doc_url(doc_url);
        }

        msg_builder.build()
    }

    fn track(&self, ctx: &RequestContext, cmd: &Command) {
        let query = String::from(cmd.query);
        let uid = ctx.uid;
        let tracker = ctx.bot_ctx.tracker.clone();

        self.pool.execute(move || {
            let msg = SearchStatsMessage { query: query.clone() };
            if let Err(err) = tracker.track(uid, "search", &msg) {
                error!("{:?}", err);
            }
        });
    }

    fn search(&self, ctx: &RequestContext, cmd: &Command) {
        let repo = self.repo.clone();
        let query = String::from(cmd.query);
        let api = ctx.bot_ctx.api.clone();
        let chat_id = ctx.chat_id;

        self.pool.execute(move || {
            let search_result = repo.search(&query);
            let send_result = match search_result {
                Ok(ref pkgs) if !pkgs.is_empty() => {
                    SearchHandler::send_packages(&api, chat_id, pkgs)
                }
                _ => SearchHandler::send_empty_result(&api, chat_id),
            };
            if let Err(err) = send_result {
                error!("{:?}", err);
            }
        });
    }
}

impl CommandHandler for SearchHandler {
    fn handle(&self, ctx: &RequestContext, cmd: &Command) -> Result<(), Error> {
        self.track(ctx, cmd);
        self.search(ctx, cmd);

        Ok(())
    }
}

pub trait Bot {
    fn get_handler<'a>(&'a self, cmd_name: &str) -> Option<&'a CommandHandler>;

    fn handle(&self, ctx: &RequestContext, text: &str) -> Result<(), Error> {
        if let Some(cmd) = Command::parse(text) {
            self.handle_cmd(ctx, &cmd)
        } else {
            Err(Error::General("Not a command".to_owned()))
        }
    }

    fn handle_cmd(&self, ctx: &RequestContext, cmd: &Command) -> Result<(), Error> {
        if let Some(ref handler) = self.get_handler(cmd.name) {
            handler.handle(ctx, cmd)
        } else {
            self.send_unrecognized_command(ctx)
        }
    }

    fn send_unrecognized_command(&self, ctx: &RequestContext) -> Result<(), Error> {
        let msg = String::from("Unrecognized command");
        try!(ctx.bot_ctx.api.send_message(ctx.chat_id, msg, Some(ParseMode::Html), None, None, None));

        Ok(())
    }

    fn run_api(&self, ctx: &BotContext) {
        let mut listener = ctx.api.listener(ListeningMethod::LongPoll(None));

        let res = listener.listen(|u| {
            if let Some(m) = u.message {
                let req_ctx = RequestContext::new(ctx.clone(), m.chat.id(), m.from.id);

                if let MessageType::Text(text) = m.msg {
                    if let Err(err) = self.handle(&req_ctx, &text) {
                            error!("{:?}", err);
                        }
                }
            }

            Ok(ListeningAction::Continue)
        });

        if let Err(e) = res {
            error!("An error occured: {}", e);
        }
    }
}

pub struct PkgsBot {
    api: Api,
    botan: Botan,
    handlers: BTreeMap<String, Box<CommandHandler>>,
}

impl PkgsBot {
    pub fn new(telegram_token: &str,
               botanio_token: &str,
               repos: Vec<(&str, Box<SearchRepo>)>)
               -> Result<PkgsBot, Error> {
        let api = try!(Api::from_token(telegram_token));
        let botan = Botan::new(botanio_token);
        let mut handlers: BTreeMap<String, Box<CommandHandler>> = BTreeMap::new();
        let pool = Arc::new(ThreadPool::new(SEARCH_WORKERS_COUNT));

        for repo in repos.into_iter() {
            let handler = SearchHandler::new(repo.1, pool.clone());
            handlers.insert(repo.0.to_owned(), Box::new(handler));
        }

        Ok(PkgsBot {
            api: api,
            botan: botan,
            handlers: handlers,
        })
    }

    pub fn run(&self) {
        let ctx = BotContext::new(&self.api, &self.botan);
        self.run_api(&ctx);
    }
}

impl Bot for PkgsBot {
    fn get_handler<'a>(&'a self, cmd_name: &str) -> Option<&'a CommandHandler> {
        self.handlers.get(cmd_name).map(|b| b.deref())
    }
}

#[test]
fn test_parse_command() {
    let data = vec![
        ("/rust abc", "rust", "abc"),
        ("/rust", "rust", ""),
        ("/rust     ", "rust", ""),
        ("/rust     a b c", "rust", "a b c"),
        ("/rust     a b c    ", "rust", "a b c")];

    for sample in &data {
        let cmd = Command::parse(sample.0).unwrap();
        assert_eq!(cmd.name, sample.1);
        assert_eq!(cmd.query, sample.2);
    }
}

#[test]
fn test_parse_invalid_command() {
    let data = vec![
        "@rust abc",
        "rust abc",
    ];

    for sample in &data {
        assert!(Command::parse(sample).is_none());
    }
}