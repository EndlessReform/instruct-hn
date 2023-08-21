use firebase_rs::Firebase;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt::{self, format};
use thiserror::Error;

pub struct FirebaseListener {
    /// TODO: Make this a connection pool if it becomes a bottleneck!
    firebase: Firebase,
    client: reqwest::Client,
    base_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Item {
    pub id: i64,
    pub deleted: Option<bool>,
    pub type_: Option<String>,
    pub by: Option<String>,
    pub time: Option<i64>,
    pub text: Option<String>,
    pub dead: Option<bool>,
    pub parent: Option<i64>,
    pub poll: Option<i64>,
    pub url: Option<String>,
    pub score: Option<i64>,
    pub title: Option<String>,
    pub parts: Option<String>,
    pub descendants: Option<i64>,
    pub kids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub created: Option<i64>,
    pub karma: Option<i64>,
    pub about: Option<String>,
    pub submitted: Option<Vec<i64>>,
}

#[derive(Error, Debug)]
pub enum FirebaseListenerErr {
    ConnectError(String),
    ParseError(String),
}

impl fmt::Display for FirebaseListenerErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FirebaseListenerErr::ConnectError(e) => write!(f, "ConnectError: {}", e),
            FirebaseListenerErr::ParseError(e) => write!(f, "ParseError: {}", e),
        }
    }
}

impl FirebaseListener {
    pub fn new(url: &str) -> Result<Self, FirebaseListenerErr> {
        let firebase = Firebase::new(url).map_err(|_| {
            FirebaseListenerErr::ConnectError(format!("Could not connect to URL {}", url))
        })?;
        let client = reqwest::Client::new();
        Ok(Self {
            firebase,
            client,
            base_url: url.to_string(),
        })
    }

    pub async fn get_item(&self, item_id: i64) -> Result<Item, FirebaseListenerErr> {
        let url = format!("{}/item/{}.json", self.base_url, item_id);
        let response = self.client.get(&url).send().await.map_err(|_| {
            FirebaseListenerErr::ConnectError(format!("Could not connect to {}", url))
        })?;

        if !response.status().is_success() {
            return Err(FirebaseListenerErr::ConnectError(format!(
                "Received unexpected status code for item {}: {}",
                item_id,
                response.status()
            )));
        }

        response
            .json::<Item>()
            .await
            .map_err(|_| FirebaseListenerErr::ParseError(format!("Item {} is not valid!", item_id)))
    }

    pub async fn get_user(&self, username: &str) -> Result<User, FirebaseListenerErr> {
        let item = self
            .firebase
            .at("user")
            .at(username)
            .get::<User>()
            .await
            .map_err(|_| {
                FirebaseListenerErr::ParseError(format!("Could not parse user {}", username))
            })?;
        Ok(item)
    }

    pub async fn get_max_id(&self) -> Result<i64, FirebaseListenerErr> {
        let max_id = self
            .firebase
            .at("maxitem")
            .get::<i64>()
            .await
            .map_err(|_| FirebaseListenerErr::ParseError("Invalid response for max item".into()))?;
        Ok(max_id)
    }
}
