use scraper::{Html, Selector};
use regex::Regex;
use super::{Compute, ComputeCount, Person, Entry};

lazy_static!{
    static ref TEASERS: Selector = Selector::parse(".teaser").unwrap();
    static ref TIME: Selector = Selector::parse(".dachzeile > a").unwrap();
    static ref NAME: Selector = Selector::parse(".headline > a").unwrap();
    static ref PARTICIPANTS: Selector = Selector::parse(".teasertext > a").unwrap();
    static ref NEXT: Selector = Selector::parse(".next > a").unwrap();
    static ref PREVIOUS: Selector = Selector::parse(".prev > a").unwrap();
    static ref BOX: Selector = Selector::parse(".box").unwrap();
    static ref PERSONS: Selector = Selector::parse(".box > h3, .box > p, .box > p > strong, .box img").unwrap();
    static ref NAMEFILTER: Regex = Regex::new(r"((?:\w|\s|-)*)(?:\((\w*)\))?").unwrap();
    static ref NAMES: Regex = Regex::new(r"Mit ((?:\w|\s|-|,)+), (?:((?:\w|\s|-)+) und )?((?:\w|\s|-)+)").unwrap();
}

const BASE_URL: &'static str = "https://daserste.ndr.de";

pub async fn scrape_webpage(url: String, results: &mut Vec<Entry>) -> Option<String>{
    let resp = reqwest::get(&url).await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();

    let fragment = Html::parse_document(&body);

    // let prev = fragment.select(&PREVIOUS).next().map(|e| e.value().attr("href").unwrap().to_owned());
    for teaser in fragment.select(&TEASERS) {
        let name = teaser.select(&NAME).to_text().unwrap().to_owned();
        let mut time_select = teaser.select(&TIME);
        let time_txt = time_select.to_text().unwrap();
        let time = chrono::NaiveDateTime::parse_from_str(&time_txt, "%d.%m.%y | %H:%M Uhr").unwrap();
        let mut participants_select = teaser.select(&PARTICIPANTS);
        let participants_txt = participants_select.to_text().unwrap();
        let url = teaser.select(&NAME).to_url(BASE_URL).unwrap();
        let detail_resp = reqwest::get(&url).await.unwrap();
        let detail_body = detail_resp.text().await.unwrap();
        let detail_html = Html::parse_document(&detail_body);
        let gaeste_url = detail_html
            .select(&PARTICIPANTS)
            .to_url(BASE_URL)
            .filter(|u| u.contains("Gaeste"));
        let participants = if let Some(gaeste_url) = gaeste_url {
            let mut participants = Vec::new();
            let gaeste_resp = reqwest::get(&gaeste_url).await.unwrap();
            let gaeste_body = gaeste_resp.text().await.unwrap();
            let gaeste_html = Html::parse_document(&gaeste_body);
            let mut name = None;
            let mut title = None;
            let mut biografie;
            let mut party = None;
            let mut image_url = None;
            for element in gaeste_html.select(&BOX).next().unwrap().select(&PERSONS) {
                if element.value().name() == "h3" {
                    if let Some(text) = element.text().next() {
                        let parts = NAMEFILTER.captures(text);
                        name = parts.get_string_count(1);
                        party = parts.get_string_count(2);
                    }
                } else if element.value().name() == "p" {
                    let is_biografie = element
                        .first_child()
                        .and_then(|n| n.value().as_element())
                        .map(|e| e.name() == "em")
                        .unwrap_or(false);
                    if is_biografie {
                        biografie = element.get_string_count(1).unwrap();
                        if biografie.len() == 0 {
                            biografie = element.get_text_count(2).unwrap_or("").to_string();
                        }
                        participants.push(Person {
                            name: name.take().unwrap(),
                            title: title.take(),
                            party: party.take(),
                            biografie: Some(biografie),
                            image_url: image_url.take(),
                        })
                    }
                } else if element.value().name() == "img" {
                    image_url = Some(format!("{}{}", BASE_URL, element.value().attr("src").unwrap()));
                } else if element.value().name() == "strong" {
                    // if <p> <strong> is used instead of <h3>
                    // see last entry in https://daserste.ndr.de/annewill/archiv/Die-Gaeste-im-Studio-,gaesteliste1200.html
                    let is_name = element
                        .prev_sibling()
                        .and_then(|c| c.value().as_element())
                        .map(|e| e.name() == "br")
                        .unwrap_or(false);
                    let text = Some(element.get_string().unwrap());
                    if is_name {
                        name = text;
                    } else {
                        title = text;
                    }
                }
            }
            participants
        } else {
            let captures = NAMES.captures(participants_txt);
            if let Some(captures) = captures {
                let mut names = Vec::new();
                for part in captures.get_text_count(1).unwrap().split(", ") {
                    names.push(part.trim().to_string());
                }
                if let Some(part) = captures.get_string_count(2) {
                    names.push(part);
                }
                names.push(captures.get_string_count(3).unwrap());
                names
                    .into_iter()
                    .map(|name| Person {
                        name,
                        title: None,
                        party: None,
                        biografie: None,
                        image_url: None,
                    })
                    .collect::<Vec<_>>()
            } else {
                let person = Person {
                    name: participants_txt.trim().to_string(),
                    title: None,
                    party: None,
                    biografie: None,
                    image_url: None,
                };
                vec![person]
            }
        };
        let entry = Entry { name, time, participants, url };
        results.push(entry);
    }

    fragment.select(&NEXT).to_url(BASE_URL)
}

pub async fn scrape(pool: &sqlx::SqlitePool) {
    let mut results = Vec::new();
    let mut url = Some("https://daserste.ndr.de/annewill/archiv/index.html".to_owned());
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
        transaction = result.write_to_db(transaction, 0).await;
    }
    transaction.commit().await.unwrap();
}
