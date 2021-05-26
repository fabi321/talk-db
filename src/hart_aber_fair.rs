use scraper::{Html, Selector};
use regex::Regex;
use super::{Compute, ComputeCount, Person, Entry};

lazy_static!{
    static ref TEASERS: Selector = Selector::parse(".teaser > a").unwrap();
    static ref TIME: Selector = Selector::parse(".teaser > a > p > .mediaDate").unwrap();
    static ref TITLE: Selector = Selector::parse(".teaser  > a > h4").unwrap();
    static ref URL: Selector = Selector::parse("a").unwrap();
    static ref PARTICIPANTS: Selector = Selector::parse(".modConStage > .modMini .teaser > a").unwrap();
    static ref SOURCE: Selector = Selector::parse("picure > source").unwrap();
    static ref NAME: Selector = Selector::parse("h4").unwrap();
    static ref DETAILS: Selector = Selector::parse("p").unwrap();
    static ref NAMEFILTER: Regex = Regex::new(r"^\s?(?:\w+\.\s)*((?:\w|\s|-)+)(?:\(\d+\))?(?:, (.+))?\s?$").unwrap();
    static ref EINZELGESPRAECH: Regex = Regex::new(r"^\s?(?:\w|\s)+:\s+((?:\w|\s|-)+)(?:\(\d+\))?\s?$").unwrap();
}

const BASE_URL: &'static str = "https://www1.wdr.de";

pub async fn scrape_webpage(url: &str, results: &mut Vec<Entry>){
    let resp = reqwest::get(url).await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();

    let fragment = Html::parse_document(&body);

    for teaser in fragment.select(&TEASERS) {
        let url = format!("{}{}", BASE_URL, teaser.value().attr("href").unwrap());
        let detail_resp = reqwest::get(&url).await.unwrap();
        assert!(detail_resp.status().is_success());
        let detail_body = detail_resp.text().await.unwrap();
        let detail_html = Html::parse_document(&detail_body);
        let mut time_txt = detail_html.select(&TIME);
        let crafted_time = format!("{} | 21:00", time_txt.to_text().unwrap_or("25.05.2020"));
        let time = chrono::NaiveDateTime::parse_from_str(&crafted_time, "%d.%m.%Y | %H:%M").unwrap();
        let name = detail_html.select(&TITLE)
            .next()
            .and_then(|e| e.text().nth(2))
            .map(|s| s.trim())
            .unwrap_or("Kinder und Eltern zuletzt - scheitern Schulen an Corona?")
            .to_owned();
        let mut participants = Vec::new();
        for participant in detail_html.select(&PARTICIPANTS) {
            let mut name = participant.select(&NAME);
            let name = name.to_text().unwrap();
            let name_parts = NAMEFILTER.captures(name)
                .or_else(|| EINZELGESPRAECH.captures(name));
            let name = name_parts.get_string_count(1).unwrap();
            let party = name_parts.get_string_count(2);
            let title = participant
                .select(&DETAILS)
                .to_text()
                .map(|s| s.replace("|", ""));
            let image_url = participant
                .select(&SOURCE)
                .next()
                .and_then(|e| e.value().attr("srcset"))
                .map(|s| format!("{}{}", BASE_URL, s));
            let participant = Person {
                name,
                title,
                party,
                biografie: None,
                image_url,
            };
            participants.push(participant);
        }
        let entry = Entry { name, time, participants, url };
        results.push(entry);
    }
}

pub async fn scrape(pool: &sqlx::SqlitePool) {
    let mut results = Vec::new();
    scrape_webpage("https://www1.wdr.de/daserste/hartaberfair/sendungen/index.html", &mut results).await;
    let mut transaction = pool.begin().await.unwrap();
    while let Some(result) = results.pop() {
        transaction = result.write_to_db(transaction, 1).await;
    }
    transaction.commit().await.unwrap();
}
