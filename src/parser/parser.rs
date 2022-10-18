use reqwest::{
    self,
    header::{ACCEPT, ACCEPT_ENCODING, COOKIE, HOST, REFERER, USER_AGENT},
};
use scraper::{ElementRef, Html, Selector};

pub struct Parser<'a> {
    pub url: &'a str,
    pub league_type: LeagueType,
    pub html: Html,
}

impl<'a> Parser<'a> {
    pub async fn new(url: &'a str, league_type: LeagueType) -> Parser {
        let client = reqwest::Client::new();

        let response = client
        .get(url)
        .header(COOKIE, "valueSRV=270; iom_consent=0000000000&1661674721436; cfid=b46516ec-af15-4bfd-8e75-7042fd37d9bd; cftoken=0")
        .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header(ACCEPT_ENCODING, "gzip, deflate, br")
        .header(HOST, "www.mytischtennis.de")
        .header(REFERER, "https://www.mytischtennis.de/clicktt/HeTTV/22-23/ligen/K31-22-23/")
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Safari/605.1.15")
        .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .send()
        .await
        .unwrap();

        match response.status() {
            reqwest::StatusCode::OK => {
                println!("Success!");
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                println!("Need to grab a new token");
            }
            _ => {
                panic!("Uh oh! Something unexpected happened.");
            }
        };

        Parser {
            url,
            league_type,
            html: Html::parse_fragment(response.text().await.unwrap().as_str()),
        }
    }

    fn extract_raw_data(&self) {
        match self.league_type {
            LeagueType::Gesamt => {
let selector = Selector::parse(r#"table[id="playingPlanDesktop"]"#).unwrap();
        let mut playing_plan_desktop: scraper::html::Select = self.html.select(&selector);

        let x = playing_plan_desktop.next().unwrap();
            }
            LeagueType::Liga => todo!(),
            LeagueType::Pokal => todo!(),
        };
        
    }
}

pub enum LeagueType {
    Liga,
    Pokal,
    Gesamt,
}
