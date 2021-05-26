use scraper::{Html, Selector};
use super::Compute;

lazy_static!{
    static ref TEASERS: Selector = Selector::parse(".b-content-teaser-item").unwrap();
    static ref NEXT: Selector = Selector::parse(".load-more-container > a").unwrap();
    static ref BRAND: Selector = Selector::parse(".teaser-cat-brand-ellipsis").unwrap();
    static ref LINK: Selector = Selector::parse("a").unwrap();
}

pub const BASE_URL: &'static str = "https://www.zdf.de";

pub async fn search(predicate: &str, results: &mut Vec<String>){
    let mut url = Some(format!("{}/suche?q={}&synth=true&sender=ZDF&contentTypes=episode", BASE_URL, predicate.replace(" ", "+")));

    while let Some(curl) = url {
        let resp = reqwest::get(&curl).await.unwrap();
        assert!(resp.status().is_success());

        let body = resp.text().await.unwrap();

        let fragment = Html::parse_document(&body);

        let mut found = false;
        
        for element in fragment.select(&TEASERS) {
            found = true;
            let mut brand_select = element.select(&BRAND);
            let brand = brand_select.to_text().unwrap();
            if brand == predicate {
                let search_url = element
                    .select(&LINK)
                    .next()
                    .unwrap()
                    .value()
                    .attr("href")
                    .unwrap();
                results.push(format!("{}{}", BASE_URL, search_url));
            }
        }

        url = if found {
            fragment
                .select(&NEXT)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|s| format!("{}{}", BASE_URL, s))
        } else {
            None
        };
    }
}

