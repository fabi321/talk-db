use scraper::{Html, Selector};
use regex::Regex;
use super::{Compute, ComputeCount, Person, Entry, zdf::{search, BASE_URL}};

lazy_static!{
    static ref TIME: Selector = Selector::parse(".other-infos .teaser-info").unwrap();
    static ref TITLE: Selector = Selector::parse(".title-wrap > .big-headline").unwrap();
    static ref PARTICIPANTS: Selector = Selector::parse(".guest-info").unwrap();
    static ref SOURCE: Selector = Selector::parse("picure > source").unwrap();
    static ref NAME: Selector = Selector::parse(".guest-name > button").unwrap();
    static ref PERSONTITILE: Selector = Selector::parse(".guest-title > p").unwrap();
    static ref DETAILS: Selector = Selector::parse(".desc-text > p").unwrap();
    static ref IMAGE: Selector = Selector::parse(".guest-img-wrap > img").unwrap();
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
        let crafted_time = format!("{} | 22:15", time_txt);
        let time = chrono::NaiveDateTime::parse_from_str(&crafted_time, "%d.%m.%Y | %H:%M").unwrap();
        let name = detail_html.select(&TITLE).to_text().unwrap().to_owned();
        let mut participants = Vec::new();
        for participant in detail_html.select(&PARTICIPANTS) {
            let mut name = participant.select(&NAME);
            let name = name.to_text().unwrap().replace("Im Einzegespräch: ", "").replace("Im Einzel-Gespräch: ", "");
            let name_parts = NAMEFILTER.captures(&name);
            let name = name_parts
                .as_ref()
                .and_then(|c| c.get(1))
                .or_else(|| name_parts.as_ref().and_then(|c| c.get(2)))
                .map(|m| m.as_str().trim())
                .unwrap_or_else(|| panic!("{} {}", name, url))
                .to_owned();
            let party = name_parts.get_string_count(2);
            let mut title_select = participant.select(&PERSONTITILE);
            let title = title_select.to_text().map(|s| s.to_owned());
            let image_url = participant
                .select(&SOURCE)
                .next()
                .and_then(|e| e.value().attr("data-src"))
                .map(|s| format!("{}{}", BASE_URL, s));
            let mut biografie_select = participant.select(&DETAILS);
            let biografie = biografie_select.to_text().map(|s| s.to_owned());
            let participant = Person {
                name,
                title,
                party,
                biografie,
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
    scrape_webpage(&mut results).await;
    let mut transaction = pool.begin().await.unwrap();
    while let Some(result) = results.pop() {
        transaction = result.write_to_db(transaction, 3).await;
    }
    transaction.commit().await.unwrap();
}
