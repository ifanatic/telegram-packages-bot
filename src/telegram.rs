use std::error::Error as StdError;
use std::iter::Iterator;
use botanio::Botan;
use marksman_escape::Escape;
use telegram_bot::{self, Api, ListeningAction, ListeningMethod, MessageType};
use telegram_bot::types::{Integer, ParseMode, User};
use error::Error;
use crates::{CratesRepo, EncodableCrate};

const MAX_MESSAGE_LENGTH: usize = 4096;

fn html_escape(s: &str) -> String {
    String::from_utf8(Escape::new(s.bytes()).collect()).unwrap()
}

struct HtmlMessageBuilder<'a> {
    name: Option<&'a str>,
    url: Option<&'a str>,
    description: Option<&'a str>,
}

impl<'a> HtmlMessageBuilder<'a> {
    fn new() -> HtmlMessageBuilder<'a> {
        HtmlMessageBuilder {
            name: None,
            url: None,
            description: None,
        }
    }

    fn name(&mut self, n: &'a str) {
        self.name = Some(n);
    }

    fn url(&mut self, u: &'a str) {
        self.url = Some(u);
    }

    fn description(&mut self, d: &'a str) {
        self.description = Some(d);
    }

    fn build(&self) -> String {
        let mut msg = String::new();

        if let Some(name) = self.name {
            let esc = html_escape(name);
            let part = format!("<b>{}</b>\n", esc);
            msg.push_str(&part);
        }

        if let Some(url) = self.url {
            let esc = html_escape(url);
            let part = format!("<a href=\"{}\">{}</a>\n", esc, esc);
            msg.push_str(&part);
        }

        if let Some(description) = self.description {
            let esc = html_escape(description);
            let part = format!("{}\n", esc);
            msg.push_str(&part);
        }

        msg
    }
}

#[derive(Debug, RustcEncodable)]
struct StatsMessage {
    query: String,
}

pub struct CratesIoBot {
    api: Api,
    crates: CratesRepo,
    botan: Botan,
}

impl CratesIoBot {
    pub fn new(telegram_token: &str, botanio_token: &str) -> Result<CratesIoBot, Error> {
        let api = try!(Api::from_token(telegram_token));
        let botan = Botan::new(botanio_token);
        Ok(CratesIoBot {
            api: api,
            crates: CratesRepo {},
            botan: botan,
        })
    }

    pub fn run(&self) {
        let mut listener = self.api.listener(ListeningMethod::LongPoll(None));

        let res = listener.listen(|u| {
            if let Some(m) = u.message {
                match m.msg {
                    MessageType::Text(t) => {
                        if t.starts_with("/search") {
                            let query = t.trim_left_matches("/search").trim();
                            if let Err(msg) = self.handle_search(m.chat.id(), String::from(query)) {
                                error!("{}", msg);
                            }

                            self.update_stats(&m.from, query);
                        }
                    }
                    _ => {}
                }
            }

            Ok(ListeningAction::Continue)
        });

        if let Err(e) = res {
            println!("An error occured: {}", e);
        }
    }

    fn render_html_message(&self, name: &str, description: &str, url: &str) -> String {
        let mut msg_builder = HtmlMessageBuilder::new();
        {
            msg_builder.name(name);
        }

        if description.len() > 0 {
            msg_builder.description(description);
        }

        if url.len() > 0 {
            msg_builder.url(url);
        }

        msg_builder.build()
    }

    fn prepare_message_text(&self, krate: &EncodableCrate) -> String {
        let description: &str = match krate.description {
            Some(ref desc) => desc,
            None => "",
        };
        let url: &str = match krate.repository {
            Some(ref u) => u,
            None => "",
        };
        self.render_html_message(&krate.name, description, url)
    }

    fn handle_search(&self, chat_id: Integer, query: String) -> telegram_bot::Result<()> {
        let crates = match self.crates.search(&query) {
            Ok(cr) => cr,
            Err(msg) => return Err(telegram_bot::Error::Api(msg.description().to_owned())),
        };

        let result = self.send_crates(chat_id, &crates);

        result
    }

    fn send_crates(&self,
                   chat_id: Integer,
                   crates: &Vec<EncodableCrate>)
                   -> telegram_bot::Result<()> {

        let mut msg = String::with_capacity(1024);

        for krate in crates.iter().take(50) {
            let msg_part = self.prepare_message_text(&krate);

            if msg.len() + msg_part.len() >= MAX_MESSAGE_LENGTH {
                break;
            }

            msg.push_str(&msg_part);
        }

        try!(self.api.send_message(chat_id, msg, Some(ParseMode::Html), None, None, None));

        Ok(())
    }

    fn update_stats(&self, user: &User, query: &str) {
        let msg = StatsMessage { query: query.to_owned() };
        if self.botan.track(user.id, "search", &msg).is_err() {
            error!("botan tracking has been failed");
        }
    }
}