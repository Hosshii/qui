use std::{
    io::{self, Read},
    path::Path,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use futures::future;
use rust_traq::{
    apis::{configuration::Configuration, notification_api},
    models::{ChannelSubscribeLevel, PutChannelSubscribeLevelRequest},
};
use tokio::time;

use super::channel;

pub async fn notify(conf: &Configuration, matches: &ArgMatches<'_>) -> Result<()> {
    if let Some(level) = matches.value_of("level") {
        let level = level
            .parse::<u8>()
            .with_context(|| "level must be 0, 1 or 2")?;
        let mut tree = channel::get_channel_tree(conf).await?;

        let ids = if let Some(ids) = matches.values_of("channel_names") {
            ids.into_iter()
                .map(|v| tree.name_to_id(Path::new(v)))
                .collect::<Result<Vec<String>>>()
                .with_context(|| "channel name not found")?
        } else {
            let mut ids = String::new();
            io::stdin().read_to_string(&mut ids)?;
            let ids: Vec<String> = ids
                .split_whitespace()
                .map(|v| tree.name_to_id(Path::new(v)))
                .collect::<Result<Vec<String>>>()
                .with_context(|| "channel name not found")?;
            ids
        };

        let ids: Vec<(String, u8)> = ids.into_iter().map(|v| (v, level)).collect();

        set_subscriptions(conf, ids).await?;
    }

    Ok(())
}

pub async fn set_subscriptions(
    conf: &Configuration,
    channel_ids_and_subscribe_level: Vec<(String, u8)>,
) -> Result<()> {
    let id_levels: Vec<(String, PutChannelSubscribeLevelRequest)> = channel_ids_and_subscribe_level
        .into_iter()
        .map(|(id, level)| {
            let level = match level {
                0 => PutChannelSubscribeLevelRequest::new(ChannelSubscribeLevel::none),
                1 => PutChannelSubscribeLevelRequest::new(ChannelSubscribeLevel::subscribed),

                2 => PutChannelSubscribeLevelRequest::new(ChannelSubscribeLevel::notified),
                _ => bail!("subscribe level must be 0, 1 or 2"),
            };
            Ok((id, level))
        })
        .collect::<Result<Vec<(String, PutChannelSubscribeLevelRequest)>>>()?;

    let mut handles = Vec::with_capacity(id_levels.len());
    for (i, (channel_id, level)) in id_levels.iter().enumerate() {
        let handle =
            notification_api::set_channel_subscribe_level(conf, channel_id, Some(level.clone()));
        handles.push(handle);

        if i % 5 == 0 {
            time::delay_for(Duration::from_millis(100)).await;
        }
    }
    let res = future::join_all(handles).await;

    res.into_iter()
        .try_for_each(|v| v)
        .with_context(|| "some error occurred")
}
