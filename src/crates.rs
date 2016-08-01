use std::io::Read;
use hyper::client::{Client, IntoUrl};
use hyper::client::response::Response;
use hyper::header::{ContentType, Headers};
use hyper::Url;
use rustc_serialize::Decodable;
use rustc_serialize::json;
use {SearchRepo, Package};
use error::Error;

const CRATES_API_ENDPOINT: &'static str = "https://crates.io/api/v1/crates";


#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct EncodableCrate {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub versions: Option<Vec<i32>>,
    pub keywords: Option<Vec<String>>,
    pub created_at: String,
    pub downloads: i32,
    pub max_version: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub links: CrateLinks,
}

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct CrateLinks {
    pub version_downloads: String,
    pub versions: Option<String>,
    pub owners: Option<String>,
    pub reverse_dependencies: String,
}

#[derive(RustcEncodable, RustcDecodable)]
struct R {
    crates: Vec<EncodableCrate>,
    meta: Meta,
}

#[derive(RustcEncodable, RustcDecodable)]
struct Meta {
    total: i64,
}

#[derive(Clone)]
pub struct CratesRepo {}

fn api_request_headers() -> Headers {
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    headers
}

fn execute_api_request<U: IntoUrl>(url: U) -> Result<Response, Error> {
    let client = Client::new();
    client.get(url).headers(api_request_headers()).send().map_err(Error::from)
}

fn from_api_response<T: Decodable>(mut resp: Response) -> Result<T, Error> {
    let mut data: String = String::new();
    try!(resp.read_to_string(&mut data));

    json::decode(&data).map_err(|e| Error::from(e))
}

impl SearchRepo for CratesRepo {
    fn search(&self, query: &str) -> Result<Vec<Package>, Error> {
        let mut endpoint = Url::parse(CRATES_API_ENDPOINT).unwrap();
        endpoint.query_pairs_mut().append_pair("q", query);
        endpoint.query_pairs_mut().append_pair("page", "1");
        endpoint.query_pairs_mut().append_pair("per_page", "50");

        let resp = try!(execute_api_request(endpoint));
        let r: R = try!(from_api_response(resp));

        let crates: Vec<Package> = r.crates
            .into_iter()
            .map(|krate| {
                Package {
                    name: krate.name,
                    repository: krate.repository,
                    documentation: krate.documentation,
                    description: krate.description,
                }
            })
            .collect();

        Ok(crates)
    }
}

#[test]
fn test_crates_search() {
    let repo = CratesRepo {};
    let crates = repo.search("telegram").unwrap();
    for krate in &crates {
        println!("{:?}", krate.name);
    }
}