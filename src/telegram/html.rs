const LONGEST_ESCAPE: usize = 6;

struct Escape<I: Iterator<Item = u8>> {
    inner: I,
    buffer: u64,
}

impl<I: Iterator<Item = u8>> Escape<I> {
    pub fn new(i: I) -> Escape<I> {
        Escape {
            inner: i,
            buffer: 0,
        }
    }
}

impl<I: Iterator<Item = u8>> Iterator for Escape<I> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.buffer != 0 {
            let ret = Some(self.buffer as u8);
            self.buffer >>= 8;
            ret
        } else if let Some(ch) = self.inner.next() {
            self.buffer = match ch {
                // Basic escapes
                b'&' => 0x3b_70_6d_61,    // amp;
                b'>' => 0x3b_74_67,       // gt;
                b'<' => 0x3b_74_6c,       // lt;
                b'"' => 0x3b_34_33_23,    // #34;
                _ => return Some(ch),
            };
            Some(b'&')
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (l, u) = self.inner.size_hint();
        (l,
         if let Some(u_) = u {
            u_.checked_mul(LONGEST_ESCAPE)
        } else {
            None
        })
    }
}


fn html_escape(s: &str) -> String {
    String::from_utf8(Escape::new(s.bytes()).collect()).unwrap()
}

struct Html {
    msg: String,
}

impl Html {
    fn new() -> Html {
        Html { msg: String::new() }
    }

    fn text<'a>(&'a mut self, t: &str) -> &'a mut Html {
        let esc = html_escape(t);
        self.msg.push_str(&esc);
        self
    }

    fn bold<'a>(&'a mut self, t: &str) -> &'a mut Html {
        let esc = html_escape(t);
        let fmt = format!("<b>{}</b>", esc);
        self.msg.push_str(&fmt);
        self
    }

    fn italic<'a>(&'a mut self, t: &str) -> &'a mut Html {
        let esc = html_escape(t);
        let fmt = format!("<i>{}</i>", esc);
        self.msg.push_str(&fmt);
        self
    }

    fn url<'a>(&'a mut self, t: &str, u: &str) -> &'a mut Html {
        let escu = html_escape(u);
        let esct = html_escape(t);
        let fmt = format!("<a href=\"{}\">{}</a>", escu, esct);
        self.msg.push_str(&fmt);
        self
    }

    fn message(&self) -> &str {
        &self.msg
    }
}


pub struct HtmlMessageBuilder<'a> {
    name: Option<&'a str>,
    repo_url: Option<&'a str>,
    doc_url: Option<&'a str>,
    description: Option<&'a str>,
}

impl<'a> HtmlMessageBuilder<'a> {
    pub fn new() -> HtmlMessageBuilder<'a> {
        HtmlMessageBuilder {
            name: None,
            repo_url: None,
            doc_url: None,
            description: None,
        }
    }

    pub fn name(&mut self, n: &'a str) {
        self.name = Some(n);
    }

    pub fn repo_url(&mut self, u: &'a str) {
        self.repo_url = Some(u);
    }

    pub fn doc_url(&mut self, u: &'a str) {
        self.doc_url = Some(u);
    }

    pub fn description(&mut self, d: &'a str) {
        self.description = Some(d);
    }

    pub fn build(&self) -> String {
        let mut html = Html::new();

        if let Some(name) = self.name {
            html.bold(name);
            html.text("\n");
        }

        if let Some(repo_url) = self.repo_url {
            html.url("[repo]", repo_url);
        }

        if let Some(doc_url) = self.doc_url {
            html.url("[doc]", doc_url);
        }

        html.text("\n");

        if let Some(description) = self.description {
            html.text(description);
            html.text("\n");
        }

        html.message().to_owned()
    }
}
