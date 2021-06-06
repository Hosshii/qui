use std::collections::{BTreeMap, HashMap};

use super::*;
use anyhow::Result;
use clap::ArgMatches;
use rust_traq::{
    apis::{self, channel_api::GetChannelsError, configuration::Configuration},
    models,
};

pub struct ChannelTree {
    node: ChannelTreeNode,
}

impl ChannelTree {
    pub fn new(node: ChannelTreeNode) -> Self {
        Self { node }
    }
}

type ChannelId = String;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelTreeNode {
    id: ChannelId,
    name: String,
    children: Vec<ChannelTreeNode>,
    active: bool,
    archived: bool,
}

impl ChannelTreeNode {
    pub fn new(
        id: ChannelId,
        name: String,
        children: Vec<ChannelTreeNode>,
        active: bool,
        archived: bool,
    ) -> Self {
        Self {
            id,
            name,
            children,
            active,
            archived,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelLike {
    id: ChannelId,
    name: String,
    parent_id: Option<ChannelId>,
    children: Vec<ChannelId>,
    archived: bool,
}

impl ChannelLike {
    pub fn new(
        id: ChannelId,
        name: impl Into<String>,
        parent_id: Option<ChannelId>,
        children: Vec<ChannelId>,
        archived: bool,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            parent_id,
            children,
            archived,
        }
    }
}

impl From<models::Channel> for ChannelLike {
    fn from(ch: models::Channel) -> Self {
        Self {
            id: ch.id,
            name: ch.name,
            parent_id: ch.parent_id,
            children: ch.children,
            archived: ch.archived,
        }
    }
}

fn construct_tree(ch: ChannelLike, mp: &BTreeMap<ChannelId, ChannelLike>) -> ChannelTreeNode {
    if ch.children.is_empty() {
        return ChannelTreeNode::new(ch.id, ch.name, Vec::new(), true, ch.archived);
    }

    let children = ch
        .children
        .into_iter()
        .filter_map(|ch_id| mp.get(&ch_id).map(|ch| construct_tree(ch.clone(), mp)))
        .collect();
    ChannelTreeNode::new(ch.id, ch.name, children, true, ch.archived)
}

pub async fn channel(conf: &Configuration, matches: &ArgMatches<'_>, cmd: &str) -> Result<()> {
    match cmd {
        "list" => {
            let tree = get_channel_tree(conf).await?;
            tree.node
                .children
                .iter()
                .for_each(|ch| println!("{}", ch.name));

            Ok(())
        }
        x => {
            dbg!("{}", x);
            Ok(())
        }
    }
}

async fn get_channel_tree(conf: &Configuration) -> Result<ChannelTree> {
    let channels = apis::channel_api::get_channels(conf, None).await.unwrap();
    let root_channel_ids: Vec<ChannelId> = channels
        .public
        .iter()
        .filter(|ch| ch.parent_id == None)
        .map(|ch| ch.id.clone())
        .collect();

    let mp: BTreeMap<ChannelId, ChannelLike> = channels
        .public
        .into_iter()
        .map(|ch| (ch.id.clone(), ChannelLike::from(ch)))
        .collect();

    let dummy_channel = ChannelLike::new("".to_owned(), "dummy", None, root_channel_ids, false);
    let tree = ChannelTree::new(construct_tree(dummy_channel, &mp));

    Ok(tree)
}
