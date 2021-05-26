use scraper::{Html, Selector};
use regex::Regex;
use super::{Compute, ComputeCount, Person, Entry, zdf::search};

lazy_static!{
    static ref TIME: Selector = Selector::parse(".other-infos .teaser-info").unwrap();
    static ref TITLE: Selector = Selector::parse(".title-wrap > .big-headline").unwrap();
    static ref PARTICIPANTS: Selector = Selector::parse(".b-post-content > .grid-container > .large-8").unwrap();
    static ref NAMEFILTER: Regex = Regex::new(r"^\s*((?:\w|\s|-)+)(?:\((.+)\))?(?:, (\w+))?\s*$").unwrap();
}

pub async fn scrape_webpage(results: &mut Vec<Entry>){
    let mut search_results = Vec::new();

    search("maybrit illner", &mut search_results).await;

    for url in search_results {
        let detail_resp = reqwest::get(&url).await.unwrap();
        assert!(detail_resp.status().is_success());
        let detail_body = detail_resp.text().await.unwrap();
        let detail_html = Html::parse_document(&detail_body);
        let mut time_txt = detail_html.select(&TIME);
        let prev_time = time_txt.next().unwrap();
        let time_txt = time_txt.next().unwrap_or(prev_time).text().next().unwrap();
        let crafted_time = format!("{} | 23:15", time_txt);
        let time = chrono::NaiveDateTime::parse_from_str(&crafted_time, "%d.%m.%Y | %H:%M").unwrap();
        let name = detail_html.select(&TITLE).to_text().unwrap().to_owned();
        let mut participants = Vec::new();
        let mut pers_name = None;
        let mut title = None;
        let mut variant = 0;
        let texts = detail_html.select(&PARTICIPANTS).next().unwrap();
        for txt in texts.text() {
            if variant == 0 {
                let name_title = NAMEFILTER.captures(txt);
                pers_name = name_title.get_string_count(1);
                title = name_title.get_string_count(2);
                variant = 1;
            } else {
                let biografie = Some(txt.to_owned());
                let person = Person {
                    name: pers_name.take().unwrap(),
                    title: title.take(),
                    biografie,
                    image_url: None,
                    party: None,
                };
                participants.push(person);
                variant = 0;
            }
        }
        let entry = Entry { name, time, participants, url };
        results.push(entry);
    }
}

pub async fn scrape(pool: &sqlx::SqlitePool) {
    let mut results = Vec::new();
    scrape_webpage(&mut results).await;
    let mut transaction = pool.begin().await.unwrap();
    while let Some(result) = results.pop() {
        transaction = result.write_to_db(transaction, 3).await;
    }
    transaction.commit().await.unwrap();
}
