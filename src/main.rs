#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate rss;

use chrono::{DateTime, FixedOffset, Utc};
use lambda::error::HandlerError;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use std::error::Error;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Episodes {
    episodes: Vec<Episode>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Episode {
    title: String,
    media_url: String,
    publication_date: String,
    shownotes: Vec<Shownote>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Shownote {
    title: String,
    url: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(handler); 

    Ok(())
}

fn handler(_e: Episodes, _c: lambda::Context) -> Result<Episodes, HandlerError> {
    let channel = Channel::from_url("http://feeds.rebuild.fm/rebuildfm").unwrap();
    let items: &[Item] = channel.items();
    let mut episodes = Episodes {
        episodes: Vec::new(),
    };
    for item in items {
        let pub_date: DateTime<FixedOffset> = DateTime::parse_from_rfc2822(&item.pub_date().unwrap()).unwrap();
        let pub_date_utc: DateTime<Utc> = pub_date.with_timezone(&Utc);
        let mut episode = Episode {
            title: item.title().unwrap().to_string(),
            media_url: item.link().unwrap().to_string(),
            publication_date: pub_date_utc.format("%+").to_string(),
            shownotes: Vec::new(),
        };

        let ul_fragment = Html::parse_fragment(item.description().unwrap());
        let ul_selector = Selector::parse("ul").unwrap();
        let li_selector = Selector::parse("li").unwrap();
        let ul = ul_fragment.select(&ul_selector).next().unwrap();
        for element in ul.select(&li_selector) {
            let a_fragment = Html::parse_fragment(&element.inner_html());
            let a_selector = Selector::parse("a").unwrap();
            let a = a_fragment.select(&a_selector).next().unwrap();
            let shownote = Shownote {
                title: a.inner_html(),
                url: a.value().attr("href").unwrap().to_string(),
            };
            episode.shownotes.push(shownote);
        }
        episodes.episodes.push(episode);
    }
    Ok(episodes)
}
