use marksman_escape::Escape;

fn html_escape(s: &str) -> String {
    String::from_utf8(Escape::new(s.bytes()).collect()).unwrap()
}

pub struct HtmlMessageBuilder<'a> {
    name: Option<&'a str>,
    url: Option<&'a str>,
    description: Option<&'a str>,
}

impl<'a> HtmlMessageBuilder<'a> {
    pub fn new() -> HtmlMessageBuilder<'a> {
        HtmlMessageBuilder {
            name: None,
            url: None,
            description: None,
        }
    }

    pub fn name(&mut self, n: &'a str) {
        self.name = Some(n);
    }

    pub fn url(&mut self, u: &'a str) {
        self.url = Some(u);
    }

    pub fn description(&mut self, d: &'a str) {
        self.description = Some(d);
    }

    pub fn build(&self) -> String {
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
