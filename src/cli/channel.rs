use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::BTreeMap,
    ffi::OsStr,
    path::{self, Path},
    rc::{Rc, Weak},
};

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use rust_traq::{
    apis::{self, configuration::Configuration},
    models,
};

pub struct ChannelTree {
    root: Rc<RefCell<ChannelTreeNode>>,
    current: Rc<RefCell<ChannelTreeNode>>,
}

impl ChannelTree {
    pub fn new(root: Rc<RefCell<ChannelTreeNode>>) -> Self {
        let current = Rc::clone(&root);
        Self { root, current }
    }

    pub fn go_path(&mut self, channel_name: &Path) -> Result<()> {
        for p in channel_name {
            if p == OsStr::new("/") {
                self.go_root();
            } else if p == OsStr::new("..") {
                self.go_up()?;
            } else {
                self.go_down(p.to_str().unwrap())?;
            }
        }
        Ok(())
    }

    fn go_root(&mut self) {
        self.current = Rc::clone(&self.root);
    }

    fn go_down(&mut self, name: &str) -> Result<()> {
        let cur;
        {
            let children = &RefCell::borrow(&self.current).children;
            let child = children
                .iter()
                .find(|ch| RefCell::borrow(ch).name == name)
                .with_context(|| format!("{} is not found", name))?;
            cur = Rc::clone(child);
        }
        self.current = cur;
        Ok(())
    }

    fn go_up(&mut self) -> Result<()> {
        let x = {
            let parent = &RefCell::borrow(&self.current).parent;

            match parent.upgrade() {
                None => {
                    bail!("parent is not exists");
                }
                Some(x) => Rc::clone(&x),
            }
        };

        self.current = x;
        Ok(())
    }
}

type ChannelId = String;
#[derive(Debug)]
pub struct ChannelTreeNode {
    id: ChannelId,
    name: String,
    children: Vec<Rc<RefCell<ChannelTreeNode>>>,
    active: bool,
    archived: bool,
    parent: Weak<RefCell<ChannelTreeNode>>,
}

impl ChannelTreeNode {
    pub fn new(
        id: ChannelId,
        name: String,
        children: Vec<Rc<RefCell<ChannelTreeNode>>>,
        active: bool,
        archived: bool,
        parent: Weak<RefCell<ChannelTreeNode>>,
    ) -> Self {
        Self {
            id,
            name,
            children,
            active,
            archived,
            parent,
        }
    }

    pub fn dummy() -> Self {
        Self {
            id: "".to_owned(),
            name: "dummy".to_owned(),
            children: Vec::new(),
            active: false,
            archived: true,
            parent: Weak::new(),
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

fn construct_tree(
    p: Weak<RefCell<ChannelTreeNode>>,
    ch: ChannelLike,
    mp: &BTreeMap<ChannelId, ChannelLike>,
) -> Rc<RefCell<ChannelTreeNode>> {
    if ch.children.is_empty() {
        let leaf = ChannelTreeNode::new(ch.id, ch.name, Vec::new(), true, ch.archived, p);
        return Rc::new(RefCell::new(leaf));
    }

    let cur = Rc::new(RefCell::new(ChannelTreeNode::new(
        ch.id,
        ch.name,
        Vec::new(),
        true,
        ch.archived,
        p,
    )));
    let children = ch
        .children
        .into_iter()
        .filter_map(|ch_id| {
            mp.get(&ch_id)
                .map(|ch| construct_tree(Rc::downgrade(&cur), ch.clone(), mp))
        })
        .collect();

    cur.borrow_mut().children = children;
    cur
}

pub async fn channel(conf: &Configuration, matches: &ArgMatches<'_>, cmd: &str) -> Result<()> {
    match cmd {
        "list" => {
            let tree = get_channel_tree(conf).await?;
            // RefCell::borrow(&tree.current)
            //     .children
            //     .iter()
            //     .for_each(|ch| println!("{}", &RefCell::borrow(&ch).name));

            Ok(())
        }
        "cd" => {
            dbg!("cd");
            if let Some(ch_name) = matches.value_of("channel_name") {
                dbg!("{}", ch_name);
                let mut tree = get_channel_tree(conf).await?;
                let path = Path::new(ch_name);
                tree.go_path(path)?;
                // RefCell::borrow(&tree.current)
                //     .children
                //     .iter()
                //     .for_each(|ch| println!("{}", &RefCell::borrow(&ch).name));
            } else {
                todo!();
            }
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
    let p = ChannelTreeNode::dummy();
    let p = Rc::downgrade(&Rc::new(RefCell::new(p)));
    let tree = ChannelTree::new(construct_tree(p, dummy_channel, &mp));

    Ok(tree)
}
