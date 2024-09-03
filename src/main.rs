use std::time::Duration;

use anyhow::{bail, Context};
use log::info;
use reqwest::Client;
use scraper::{selectable::Selectable, Html};

use clubdam_shopsearch::{
    regex, selector, CityCode, Machines, PrefCode, Recordings, Scorings, Store,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let client = Client::new();

    for pref in PrefCode::iter() {
        for city in get_city_list(&client, pref).await? {
            let res = process_city(&client, pref, city).await?;
            for res in res {
                println!("{pref} {city} {res:?}");
            }
        }
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

async fn process_city(
    client: &Client,
    pref: PrefCode,
    city: CityCode,
) -> anyhow::Result<Vec<Store>> {
    info!("Processing prefecture {pref}, city {city}");
    let html = Html::parse_document(
        &client
            .get(format!(
                "https://www.clubdam.com/shopsearch/?todofukenCode={pref}&cityCode={city}"
            ))
            .send()
            .await?
            .text()
            .await?,
    );
    sleep(Duration::from_secs_f64(0.5)).await;
    html.select(selector!("li.result-item"))
        .map(|e| {
            (|| {
                let name = e
                    .select(selector!("span.store-name,a.store-name"))
                    .next()
                    .context("Missing store name")?
                    .text()
                    .collect();
                let address = e
                    .select(selector!("div.store-address > span"))
                    .next()
                    .context("Missing address")?
                    .text()
                    .collect();
                let map = regex!(r"https://www.google.co.jp/maps\?q=([0-9.-]+),([0-9.-]+)")
                    .captures(
                        e.select(selector!("div.store-address > a"))
                            .next()
                            .context("Missing map link")?
                            .attr("href")
                            .context("Missing href in map link")?,
                    )
                    .context("Invalid map link format")?;
                let latitude = map[1].parse().context("Invalid latitude format")?;
                let longitude = map[2].parse().context("Invalid latitude format")?;
                let phone = e
                    .select(selector!("div.store-tel > a"))
                    .next()
                    .context("Missing store telephone number")?
                    .text()
                    .collect();
                let url = e
                    .select(selector!("div.store-url > a"))
                    .next()
                    .map(|e| {
                        anyhow::Ok(
                            e.attr("href")
                                .context("Missing href in store url")?
                                .parse()
                                .context("Invalid store URL")?,
                        )
                    })
                    .transpose()?;
                let mut machines = Machines::default();
                for e in e.select(selector!("li.machine-item > img")) {
                    match e.attr("alt").context("Missing alt in machine-item img")? {
                        "LIVE DAM Ai" => machines.ai = true,
                        "LIVE DAM STADIUM" => machines.studium = true,
                        "LIVE DAM" => machines.normal = true,
                        "PremierDAM" => machines.premier = true,
                        // Cyber dam should be somewhere...
                        e => bail!("Unexpected machine: {e:?}"),
                    }
                }
                let mut recordings = Recordings::default();
                let mut scorings = Scorings::default();
                for e in e.select(selector!("li.feature-item > span")) {
                    match &e.text().collect::<String>()[..] {
                        "DAM★とも動画" => recordings.video = true,
                        "DAM★とも録音" => recordings.voice = true,
                        "精密採点Ai" => scorings.ai = true,
                        "精密採点DX-G" => scorings.dx_g = true,
                        "精密採点DX" => scorings.dx = true,
                        e => bail!("Unexpected machine: {e:?}"),
                    }
                }
                anyhow::Ok(Store {
                    name,
                    address,
                    latitude,
                    longitude,
                    phone,
                    url,
                    machines,
                    recordings,
                    scorings,
                })
            })()
            .with_context(|| format!("While parsing {:?} in pref={pref}, city={city}", e.html()))
        })
        .collect()
}
