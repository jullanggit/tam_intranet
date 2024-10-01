use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use compact_str::CompactString;
use regex_lite::Regex;
use reqwest::{cookie::Jar, header::HeaderMap};

use crate::{resources::Name, CSRF_REGEX, DEBUG};

pub struct Authenticated;
pub struct Unauthenticated;

pub struct IntranetClient<State> {
    client: reqwest::Client,
    jar: Arc<Jar>,
    school: School,
    pub student: Name,
    _state: State,
}
impl IntranetClient<Unauthenticated> {
    /// student: (firstname.lastname)
    pub fn new(school: School, student: impl Into<CompactString>) -> Result<Self> {
        let jar = Arc::new(Jar::default());

        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:87.0) Gecko/20100101 Firefox/87.0".parse()?,
        );

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .cookie_provider(jar.clone())
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            client,
            jar,
            school,
            student: Name {
                string: student.into(),
            },
            _state: Unauthenticated,
        })
    }
    pub async fn authenticate(self, password: &str) -> Result<IntranetClient<Authenticated>> {
        let hash = self.get_hash().await?;

        let mut auth_form = HashMap::new();
        auth_form.insert("hash", hash.as_ref());
        auth_form.insert("loginschool", self.school.code());
        auth_form.insert("loginuser", self.student.string.as_str());
        auth_form.insert("loginpassword", password);

        self.client
            .post(self.school_url())
            .form(&auth_form)
            .send()
            .await?
            .error_for_status()?;

        Ok(IntranetClient {
            client: self.client,
            jar: self.jar,
            school: self.school,
            student: self.student,
            _state: Authenticated,
        })
    }

    /// Extracts the hash field from the intranet website (necessary for authenticating)
    async fn get_hash(&self) -> Result<String> {
        let response_body = self.client.get(self.url()).send().await?.text().await?;
        // Parse to HTML
        let html_dom = tl::parse(&response_body, tl::ParserOptions::default())?;

        let hash = html_dom
            .query_selector("input[name='hash']")
            .unwrap()
            .next()
            .and_then(|node| node.get(html_dom.parser()).unwrap().as_tag())
            .and_then(|tag| tag.attributes().get("value").flatten())
            .unwrap()
            .as_utf8_str()
            .to_string();

        if DEBUG {
            println!("Hash: {hash}");
        }

        Ok(hash)
    }
}
impl<State> IntranetClient<State> {
    pub fn school_url(&self) -> String {
        format!("{}/{}", self.url(), self.school.code())
    }
    pub fn url(&self) -> &'static str {
        "https://intranet.tam.ch"
    }
    pub fn school(&self) -> School {
        self.school
    }
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
    /// TODO: Check if needs authentication
    pub async fn get_csrf_token(&self, url: &str) -> Result<String> {
        let classbook_response = self.client().get(url).send().await?.text().await?;
        let csrf_token = Regex::new(CSRF_REGEX)
            .unwrap()
            .captures(&classbook_response)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();

        if DEBUG {
            dbg!(csrf_token);
        }

        Ok(csrf_token.to_string())
    }
}

#[derive(Clone, Copy)]
pub enum School {
    Mng,
}
impl School {
    const fn code(&self) -> &str {
        match self {
            Self::Mng => "krm",
        }
    }
}
