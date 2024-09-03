use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};
use url::Url;

#[macro_export]
macro_rules! selector {
    ($e: expr) => {{
        use ::once_cell::sync::Lazy;
        use ::scraper::Selector;
        static SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse($e).unwrap());
        &*SELECTOR
    }};
}

#[macro_export]
macro_rules! regex {
    ($e: expr) => {{
        use ::once_cell::sync::Lazy;
        use ::regex::Regex;
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new($e).unwrap());
        &*PATTERN
    }};
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PrefCode(u8);
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
impl PrefCode {
    pub fn iter() -> impl Iterator<Item = PrefCode> {
        (1..=47).map(PrefCode)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, FromStr, Display, Serialize, Deserialize)]
pub struct CityCode(u32);

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    pub prefecture: PrefCode,
    pub city: CityCode,
    pub name: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub phone: String,
    pub url: Option<Url>,
    pub machines: Machines,
    pub recordings: Recordings,
    pub scorings: Scorings,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Machines {
    pub ai: bool,
    pub studium: bool,
    pub normal: bool,
    pub premier: bool,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Recordings {
    pub video: bool,
    pub voice: bool,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Scorings {
    pub ai: bool,
    pub dx_g: bool,
    pub dx: bool,
}
