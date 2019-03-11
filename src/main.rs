#[macro_use] extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate rss;

use rss::{Channel, Item};
use scraper::{Html, Selector};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Episodes {
    episodes: Vec<Episode>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Episode {
    title: String,
    media_url: String,
    publication_date: String,
    shownotes: Vec<Shownote>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Shownote {
    title: String,
    url: String,
}

fn main() -> Result<(), Box<std::error::Error>> {
    let channel = Channel::from_url("http://feeds.rebuild.fm/rebuildfm")?;
    let items: &[Item] = channel.items();
    let mut episodes = Episodes {
        episodes: Vec::new(),
    };
    for item in items {
        let mut episode = Episode {
            title: item.title().unwrap().to_string(),
            media_url: item.link().unwrap().to_string(),
            publication_date: item.pub_date().unwrap().to_string(),
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
        // break;
    }
    let json_str = serde_json::to_string(&episodes);
    println!("{:?}", json_str.unwrap());
    Ok(())
}
