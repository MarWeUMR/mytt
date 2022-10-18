use base64ct::{Base64, Encoding};
use polars::prelude::*;
use reqwest::{
    self,
    header::{ACCEPT, ACCEPT_ENCODING, COOKIE, HOST, REFERER, USER_AGENT},
};
use sha2::{Digest, Sha256};

use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.mytischtennis.de/clicktt/HeTTV/22-23/verein/34031/TV-1861-Wallau/spielplan/")
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

    let document = Html::parse_fragment(response.text().await.unwrap().as_str());

    let selector = Selector::parse(r#"table[id="playingPlanDesktop"]"#).unwrap();
    let mut playing_plan_desktop: scraper::html::Select = document.select(&selector);

    let x = playing_plan_desktop.next().unwrap();

    let rows = extract_rows_from_table(x);

    let mut v: Vec<Entry> = Vec::new();

    let mut backup_date = String::new().into_boxed_str();
    rows.into_iter().skip(1).for_each(|row| {
        let y: Entry = extract_column_data_from_row(row, &mut backup_date);

        v.push(y);
    });

    // we need to serialize the vector of entrys as json
    // in order to create a DataFrame from it, for which
    // the schema is inferred from the json.
    let json = serde_json::to_string(&v).unwrap();
    let cursor = std::io::Cursor::new(json);

    // 4. Create polars DataFrame from reading cursor as json
    let df: DataFrame = JsonReader::new(cursor).finish().unwrap();

    println!("{:?}", df);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Entry {
    // TODO: datum als echtes datum machen
    // TODO: implement some sort of info collection e.g. hallentausch etc.
    uhrzeit: String,
    datum: String,
    halle: String,
    liga: String,
    heim: String,
    gast: String,
    ergebnis: String,
    hash_id: String,
}

impl Entry {
    fn new(
        uhrzeit: String,
        datum: String,
        halle: String,
        liga: String,
        heim: String,
        gast: String,
        ergebnis: String,
    ) -> Entry {
        let mut hasher = Sha256::new();
        hasher.update(&liga);
        hasher.update(&heim);
        hasher.update(&gast);
        let hash = hasher.finalize();
        let base64_hash = Base64::encode_string(&hash);

        Entry {
            uhrzeit,
            datum,
            halle,
            liga,
            heim,
            gast,
            ergebnis,
            hash_id: base64_hash,
        }
    }
}

fn extract_rows_from_table(table: ElementRef) -> Vec<ElementRef> {
    let selector = Selector::parse("tr").unwrap();
    let rows = table.select(&selector);
    let x = rows.collect::<Vec<_>>();
    x
}

/// This method extracts the column data from a row.
/// If there is no date set, the backup from the last known date is used.
/// Otherwise the backup is updated.
/// An entry is returned.
fn extract_column_data_from_row<'a>(row: ElementRef<'a>, backup_date: &mut Box<str>) -> Entry {
    let selector = Selector::parse("td").unwrap();
    let cols = row.select(&selector);
    let x = cols.collect::<Vec<_>>();

    let datum = match x[0].text().into_iter().nth(1) {
        Some(s) => {
            *backup_date = s.to_string().into_boxed_str();
            String::from(s)
        }
        None => backup_date.to_string(),
    };

    let ergebnis = match x[8].text().into_iter().nth(1) {
        Some(s) => s,
        None => "-",
    };

    let halle = match x[2].text().into_iter().nth(1) {
        Some(s) => s,
        // TODO: if an error happens here, collect that info
        None => "1",
    };

    let e = Entry::new(
        datum.to_string().replace("\n", ""),
        x[1].text().next().unwrap().to_string().replace("\n", ""),
        halle.to_string().replace("\n", ""),
        x[3].text().next().unwrap().to_string(),
        x[4].text().into_iter().nth(1).unwrap().to_string(),
        x[5].text().into_iter().nth(1).unwrap().to_string(),
        ergebnis.to_string(),
    );
    println!("{:?}", &e);
    e
}
