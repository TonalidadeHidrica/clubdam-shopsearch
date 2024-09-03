use std::time::Duration;

use anyhow::{bail, Context};
use derive_more::{Display, From, FromStr};
use log::info;
use reqwest::Client;
use scraper::Html;

use clubdam_shopsearch::{regex, selector};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let client = Client::new();

    for i in 1..=47 {
        let cities = get_city_list(&client, i.into()).await?;
        println!("{cities:?}");
    }
    Ok(())
}

async fn get_city_list(client: &Client, code: PrefCode) -> anyhow::Result<Vec<CityCode>> {
    info!("Processing prefecture {code}");
    let html = Html::parse_document(
        &client
            .get(format!(
                "https://www.clubdam.com/shopsearch/?todofukenCode={code}"
            ))
            .send()
            .await?
            .text()
            .await?,
    );
    sleep(Duration::from_secs_f64(0.5)).await;
    html.select(selector!(r#"a[href*="?todofukenCode="]"#))
        .map(|e| {
            (|| {
                let captures = regex!(
                    r#"(?x)
                    \./\?
                    todofukenCode = (?<pref> \d+ ) &
                    cityCode = (?<city> \d+ )
                "#
                )
                .captures(e.attr("href").context("href not found")?)
                .context("No match")?;
                let pref: PrefCode = captures
                    .name("pref")
                    .unwrap()
                    .as_str()
                    .parse()
                    .context("Failed to parse pref")?;
                if pref != code {
                    bail!("Wrong prefecture code: expected {code}, found {pref}");
                }
                let city: CityCode = captures
                    .name("city")
                    .unwrap()
                    .as_str()
                    .parse()
                    .context("Failed to parse city")?;
                anyhow::Ok(city)
            })()
            .with_context(|| format!("While parsing {:?}", e.html()))
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, From)]
struct PrefCode(u8);
impl std::fmt::Display for PrefCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", self.0)
    }
}
impl std::str::FromStr for PrefCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        Ok(Self(s.strip_prefix('0').unwrap_or(s).parse()?))
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, FromStr, Display)]
struct CityCode(u32);
