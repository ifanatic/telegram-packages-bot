use std::io::{self, Read};
use hyper::client::Client;
use hyper::Url;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name};
use tendril::{ByteTendril, ReadExt};
use {SearchRepo, Package};
use error;

const GODOC_ENDPOINT: &'static str = "http://godoc.org";

#[derive(Clone)]
pub struct GodocRepo {}

impl GodocRepo {
    fn parse_page<R: Read>(&self, r: R) -> Result<Vec<Package>, error::Error> {
        let doc = try!(self.document_from_read(r));

        match doc.find(Name("tbody")).first() {
            Some(ref n) => Ok(self.parse_packages_table(n)),
            None => Err(error::Error::General("Invalid godoc page".to_owned())),
        }
    }

    fn document_from_read<R: Read>(&self, mut readable: R) -> io::Result<Document> {
        let mut byte_tendril = ByteTendril::new();
        try!(readable.read_to_tendril(&mut byte_tendril));

        match byte_tendril.try_reinterpret() {
            Ok(str_tendril) => Ok(Document::from(str_tendril)),
            Err(_) => {
                Err(io::Error::new(io::ErrorKind::InvalidData,
                                   "stream did not contain valid UTF-8"))
            }
        }
    }

    fn parse_packages_table<'a>(&self, tbody: &Node<'a>) -> Vec<Package> {
        tbody.find(Name("tr"))
            .iter()
            .filter_map(|n| self.parse_packages_row(&n))
            .collect::<Vec<Package>>()
    }

    fn parse_packages_row<'a>(&self, tr: &Node<'a>) -> Option<Package> {
        let mut pkg: Package = Package::empty();

        if let Some(n) = tr.find(Name("a")).first() {
            pkg.name = n.text();

            if let Some(repourl) = n.attr("href") {
                let repo = repourl.trim_left_matches('/');

                let docurl = format!("http://godoc.org/{}", repo);
                pkg.repository = Some(repo.to_owned());
                pkg.documentation = Some(docurl);
            }
        } else {
            return None;
        }

        if let Some(n) = tr.find(Class("synopsis")).first() {
            pkg.description = Some(n.text());
        } else {
            return None;
        }

        if !pkg.name.is_empty() {
            Some(pkg)
        } else {
            None
        }
    }
}

impl SearchRepo for GodocRepo {
    fn search(&self, query: &str) -> Result<Vec<Package>, error::Error> {
        let mut endpoint = Url::parse(GODOC_ENDPOINT).unwrap();
        endpoint.query_pairs_mut().append_pair("q", query);;

        let client = Client::new();
        let resp = try!(client.get(endpoint).send());

        self.parse_page(resp)
    }
}

#[test]
fn test_godoc_search() {
    let repo = GodocRepo {};
    let packages = repo.search("test").unwrap();
    assert!(packages.len() > 0);

    for pkg in &packages {
        println!("{:?}, {:?}, {:?}",
                 pkg.name,
                 pkg.description,
                 pkg.repository);
    }
}