use firebase_rs::Firebase;
use flume::{SendError, Sender};
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt::{self};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

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

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateData {
    pub items: Option<Vec<i64>>,
    pub profiles: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Update {
    pub path: String,
    pub data: UpdateData,
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
    JsonParseError(#[from] serde_json::Error), // Added for JSON parsing errors
    ChannelError(#[from] SendError<i64>),
}

impl fmt::Display for FirebaseListenerErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FirebaseListenerErr::ConnectError(e) => write!(f, "ConnectError: {}", e),
            FirebaseListenerErr::ParseError(e) => write!(f, "ParseError: {}", e),
            FirebaseListenerErr::JsonParseError(e) => write!(f, "ParseError: {}", e),
            FirebaseListenerErr::ChannelError(e) => write!(f, "ChannelError: {}", e),
        }
    }
}

impl FirebaseListener {
    pub fn new(url: String) -> Result<Self, FirebaseListenerErr> {
        let firebase = Firebase::new(&url).map_err(|_| {
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

    pub async fn listen_to_updates(
        &self,
        tx: Sender<i64>,
        cancel_token: CancellationToken,
    ) -> Result<(), FirebaseListenerErr> {
        let mut stream = self
            .firebase
            .at("updates")
            .with_realtime_events()
            .ok_or(FirebaseListenerErr::ConnectError(format!(
                "Could not connect to events for {}",
                &self.base_url
            )))? // Handle connection error
            .stream(true);

        loop {
            tokio::select! {
                event_option = stream.next() => {
                    if let Some(event) = event_option {
                        match event {
                            Ok((event_type, maybe_data)) => match maybe_data {
                                Some(s) => match serde_json::from_str::<Update>(&s) {
                                Ok(update) => {
                                    if let Some(ids) = update.data.items {
                                        info!("{:?}; {:?} new items", event_type, ids.len());
                                        debug!("{:?}", ids);
                                        for id in ids {
                                            tx.send_async(id).await?;
                                        }
                                    }
                                }
                                Err(err) => {
                                    error!("Error parsing JSON for event {:?}: {:?}", event_type, err);
                                }
                                },
                                None => warn!("{:?} {:?}", event_type, maybe_data)
                            }
                            Err(err) => {
                                error!("Error {:?}", err);
                            }
                        }
                    } else {
                        // Stream ended
                        break;
                    }
                }
                _ = cancel_token.cancelled() => {
                    info!("Cancellation token triggered, exiting listen_to_updates.");
                    break;
                }
            }
        }
        Ok(())
    }
}
