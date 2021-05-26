use scraper::{Html, Selector};
use regex::Regex;
use super::{Compute, ComputeCount, Person, Entry};

lazy_static!{
    static ref TEASERS: Selector = Selector::parse(".viewA").unwrap();
    static ref TIME: Selector = Selector::parse(".ressort").unwrap();
    static ref NAME: Selector = Selector::parse(".headline > a").unwrap();
    static ref TIMEFILTER: Regex = Regex::new(r"\d{2}\.\d{2}\.\d{4}").unwrap();
    static ref PARTICIPANTS: Selector = Selector::parse("p").unwrap();
    static ref NEXT: Selector = Selector::parse(".right > a").unwrap();
    static ref STRONG: Selector = Selector::parse("strong").unwrap();
    static ref BOX: Selector = Selector::parse(".box").unwrap();
    static ref PERSONS: Selector = Selector::parse(".box > h3, .box > p, .box > p > strong, .box img").unwrap();
    static ref NAMEFILTER: Regex = Regex::new(r"((?:\w|\s|-)+)(?:, (.+) )?\((.*)\)").unwrap();
}

const BASE_URL: &'static str = "https://www.daserste.de";

pub async fn scrape_webpage(url: String, results: &mut Vec<Entry>) -> Option<String>{
    let resp = reqwest::get(&url).await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();

    let fragment = Html::parse_document(&body);

    for teaser in fragment.select(&TEASERS) {
        let name = teaser.select(&NAME).to_text().unwrap().to_owned();
        let mut time_select = teaser.select(&TIME);
        let time_txt = time_select.to_text().unwrap();
        let time_txt = TIMEFILTER.find(time_txt).unwrap().as_str();
        let time = chrono::NaiveDateTime::parse_from_str(&format!("{} | 22:50", time_txt), "%d.%m.%Y | %H:%M").unwrap();
        let url = teaser.select(&NAME).to_url(BASE_URL).unwrap();
        let detail_resp = reqwest::get(&url).await.unwrap();
        let detail_body = detail_resp.text().await.unwrap();
        let detail_html = Html::parse_document(&detail_body);
        let mut participants = Vec::new();
        for element in detail_html.select(&PARTICIPANTS) {
            let strong = element.first_child()
                .and_then(|n| n.value().as_element())
                .map(|e| e.name() == "strong")
                .unwrap_or(false);
            if strong {
                let mut name = "".to_owned();
                for text in element.text() {
                    name.push_str(text);
                }
                let name = name.trim();
                if name.contains("Gäste") || name.is_empty() {
                    continue;
                }
                let captures = NAMEFILTER.captures(&name).unwrap_or_else(||{println!("{}",name);panic!()});
                let name = captures.get_string_count(1).unwrap();
                let party = captures.get_string_count(2);
                let title = captures.get_string_count(3);
                let person = Person {
                    name,
                    title,
                    party,
                    biografie: None,
                    image_url: None
                };
                participants.push(person);
            }
        }
        let entry = Entry { name, time, participants, url };
        results.push(entry);
    }

    fragment.select(&NEXT).to_url(BASE_URL)
}

pub async fn scrape(pool: &sqlx::SqlitePool) {
    let mut results = Vec::new();
    let mut url = Some("https://www.daserste.de/information/talk/maischberger/sendung/index.html".to_owned());
    let mut transaction = pool.begin().await.unwrap();
    while let Some(curl) = url {
        url = scrape_webpage(curl, &mut results).await;
        let res = sqlx::query_file!("src/select_url.sql", results.last().unwrap().url)
            .fetch_optional(&mut transaction)
            .await
            .unwrap();
        if let Some(_) = res {
            url = None
        }
    }
    while let Some(result) = results.pop() {
        transaction = result.write_to_db(transaction, 2).await;
    }
    transaction.commit().await.unwrap();
}
