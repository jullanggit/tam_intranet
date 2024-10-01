use crate::{resources::Name, CSRF_REGEX, DEBUG};
use anyhow::Result;
use compact_str::CompactString;
use cookie_store::CookieStore;
use regex_lite::Regex;
use std::time::Duration;

pub struct Authenticated;
pub struct Unauthenticated;

pub struct IntranetClient<State> {
    pub client: ureq::Agent,
    school: School,
    pub student: Name,
    _state: State,
}
impl IntranetClient<Unauthenticated> {
    /// student: (firstname.lastname)
    pub fn new(school: School, student: impl Into<CompactString>) -> Result<Self> {
        let cookie_store = CookieStore::new(None);
        let client = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .https_only(true)
            .cookie_store(cookie_store)
            .build();

        Ok(Self {
            client,
            school,
            student: Name {
                string: student.into(),
            },
            _state: Unauthenticated,
        })
    }
    pub fn authenticate(self, password: &str) -> Result<IntranetClient<Authenticated>> {
        let hash = self.get_hash()?;

        let auth_form = [
            ("hash", hash.as_ref()),
            ("loginschool", self.school.code()),
            ("loginuser", self.student.string.as_str()),
            ("loginpassword", password),
        ];

        self.client
            .post(&self.school_url())
            // TODO: See if i have to do this everywhere
            .set(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:87.0) Gecko/20100101 Firefox/87.0",
            )
            .send_form(&auth_form)?;

        Ok(IntranetClient {
            client: self.client,
            school: self.school,
            student: self.student,
            _state: Authenticated,
        })
    }

    /// Extracts the hash field from the intranet website (necessary for authenticating)
    fn get_hash(&self) -> Result<String> {
        let response_body = self.client.get(self.url()).call()?.into_string()?;

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
    /// TODO: Check if needs authentication
    pub fn get_csrf_token(&self, url: &str) -> Result<String> {
        let classbook_response = self.client.get(url).call()?.into_string()?;
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
    UetikonAmSee,
}
impl School {
    const fn code(&self) -> &str {
        match self {
            Self::Mng => "krm",
            Self::UetikonAmSee => "kue",
        }
    }
}
