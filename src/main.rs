use anyhow::Result;
use once_cell::sync::Lazy;
use std::{
    path::{Path, PathBuf, MAIN_SEPARATOR},
    sync::RwLock,
};
use watchexec::{action::ActionHandler, error::RuntimeError, filter::Filterer, Config, Watchexec};
use watchexec_events::{
    filekind::{FileEventKind, ModifyKind},
    Event, Priority, Tag,
};
use watchexec_filterer_globset::GlobsetFilterer;

static IGNORES: Lazy<RwLock<Vec<(String, Option<PathBuf>)>>> = Lazy::new(|| {
    vec![
        (format!("**{MAIN_SEPARATOR}.DS_Store"), None),
        (String::from("watchexec.*.log"), None),
        (String::from("*.py[co]"), None),
        (String::from("#*#"), None),
        (String::from(".#*"), None),
        (String::from(".*.kate-swp"), None),
        (String::from(".*.sw?"), None),
        (String::from(".*.sw?x"), None),
        (format!("**{MAIN_SEPARATOR}.bzr{MAIN_SEPARATOR}**"), None),
        (format!("**{MAIN_SEPARATOR}_darcs{MAIN_SEPARATOR}**"), None),
        (
            format!("**{MAIN_SEPARATOR}.fossil-settings{MAIN_SEPARATOR}**"),
            None,
        ),
        (format!("**{MAIN_SEPARATOR}.git{MAIN_SEPARATOR}**"), None),
        (format!("**{MAIN_SEPARATOR}.hg{MAIN_SEPARATOR}**"), None),
        (format!("**{MAIN_SEPARATOR}.pijul{MAIN_SEPARATOR}**"), None),
        (format!("**{MAIN_SEPARATOR}.svn{MAIN_SEPARATOR}**"), None),
    ]
    .into()
});

#[tokio::main]
async fn main() -> Result<()> {
    let anction_handler = |action: ActionHandler| {
        for event in action.events.iter() {
            println!("EVENT: {event:?}");
        }
        println!("\n\n\n");
        action
    };

    let ignores = IGNORES.read().unwrap().clone();

    let origin = Path::new(".");
    let filterer = WatchexecFilterer {
        inner: GlobsetFilterer::new(origin, vec![], ignores, vec![], vec![]).await?,
        fs_events: vec![
            FsEvent::Access,
            FsEvent::Modify,
            FsEvent::Create,
            FsEvent::Remove,
        ],
    };

    let config = Config::default();
    let config = config
        // .pathset(["/Volumes/Data exFAT/Temp/src", "."])
        .pathset(["/Volumes/Data exFAT/Temp/src"])
        .filterer(filterer)
        .on_action(anction_handler);

    match Watchexec::with_config(config.clone()) {
        Ok(wx) => {
            if let Err(err) = wx.main().await {
                eprintln!("Error: {err}");
            }
        }
        Err(err) => {
            eprintln!("Critical Error: {err}");
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct WatchexecFilterer {
    inner: GlobsetFilterer,
    fs_events: Vec<FsEvent>,
}

impl Filterer for WatchexecFilterer {
    fn check_event(&self, event: &Event, priority: Priority) -> Result<bool, RuntimeError> {
        for tag in &event.tags {
            if let Tag::FileEventKind(fek) = tag {
                let normalised = match fek {
                    FileEventKind::Access(_) => FsEvent::Access,
                    FileEventKind::Modify(ModifyKind::Name(_)) => FsEvent::Rename,
                    FileEventKind::Modify(ModifyKind::Metadata(_)) => FsEvent::Metadata,
                    FileEventKind::Modify(_) => FsEvent::Modify,
                    FileEventKind::Create(_) => FsEvent::Create,
                    FileEventKind::Remove(_) => FsEvent::Remove,
                    _ => continue,
                };

                if !self.fs_events.contains(&normalised) {
                    return Ok(false);
                }
            }
        }

        if !self.inner.check_event(event, priority)? {
            return Ok(false);
        }

        Ok(true)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsEvent {
    Access,
    Create,
    Remove,
    Rename,
    Modify,
    Metadata,
}
