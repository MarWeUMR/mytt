use reqwest::{
    self,
    header::{ACCEPT, ACCEPT_ENCODING, COOKIE, HOST, REFERER, USER_AGENT},
};

use scraper::{ElementRef, Html, Selector};

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
    let playing_plan_desktop: scraper::html::Select = document.select(&selector);

    let x = playing_plan_desktop.into_iter().nth(0).unwrap();

    let rows = extract_rows_from_table(x);

    let mut backup_date = "";
    rows.into_iter().skip(1).for_each(|row| {
        let y = extract_column_data_from_row(row, backup_date);
        println!("{:?}", y);
    });
}

#[derive(Debug)]
struct Entry {
    uhrzeit: String,
    datum: String,
    halle: String,
    liga: String,
    heim: String,
    gast: String,
    ergebnis: String,
}

fn extract_rows_from_table(table: ElementRef) -> Vec<ElementRef> {
    let selector = Selector::parse("tr").unwrap();
    let rows = table.select(&selector);
    let x = rows.collect::<Vec<_>>();
    x
}

fn extract_column_data_from_row(row: ElementRef, mut backup_date: &str) -> Entry {
    let selector = Selector::parse("td").unwrap();
    let cols = row.select(&selector);
    let x = cols.collect::<Vec<_>>();
    println!("{:?}", x[0].html().as_str());

    let datum = match x[0].text().into_iter().nth(1) {
        Some(s) => s,
        // TODO: hier muss noch das backup datum rein
        None => "b",
    };

    let ergebnis = match x[8].text().into_iter().nth(1) {
        Some(s) => s,
        None => "-",
    };

    let halle = match x[2].text().into_iter().nth(1) {
        Some(s) => s,
        None => "1?",
    };

    let e = Entry {
        // TODO: Backup needed here, if the cell is empty.
        datum: datum.to_string(),
        uhrzeit: x[1].text().into_iter().nth(0).unwrap().to_string(),
        halle: halle.to_string(),
        liga: x[3].text().into_iter().nth(0).unwrap().to_string(),
        heim: x[4].text().into_iter().nth(1).unwrap().to_string(),
        gast: x[5].text().into_iter().nth(1).unwrap().to_string(),
        ergebnis: ergebnis.to_string(),
    };
    e
}
