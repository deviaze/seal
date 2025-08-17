use mluau::prelude::*;
use crate::prelude::*;
use crate::std_fs::pathlib::normalize_path;
use std::{path::Path, time::Duration};
use std::sync::{Arc, Mutex};

use notify::{event::{
    AccessKind, AccessMode, 
    CreateKind, DataChange, 
    MetadataKind, ModifyKind, 
    RemoveKind, RenameMode}, 
    Event, EventKind, 
    RecursiveMode, Watcher
};
use crossbeam_channel::RecvTimeoutError;

#[derive(Clone, Copy)]
pub struct WatchOptions {
    recursive: bool,
    timeout: Duration,
}
impl WatchOptions {
    pub fn default() -> Self {
        Self {
            recursive: true,
            timeout: Duration::from_millis(10),
        }
    }
    pub fn from_table(t: LuaTable, function_name: &'static str) -> LuaResult<Self> {
        let recursive = match t.raw_get("recursive")? {
            LuaValue::Boolean(b) => b,
            LuaNil => true,
            other => {
                return wrap_err!("{} expected WatchOptions.recursive to be a boolean (default true), got: {:?}", function_name, other);
            }
        };
        let timeout = match t.raw_get("timeout_ms")? {
            LuaValue::Integer(i) => {
                Duration::from_millis(int_to_u64(i, function_name, "timeout_ms")?)
            },
            LuaValue::Number(f) => {
                Duration::from_millis(float_to_u64(f, function_name, "timeout_ms")?)
            },
            LuaNil => Duration::from_millis(10),
            other => {
                return wrap_err!("{} expected WatchOptions.timeout_ms to be a number (in milliseconds) or nil, got: {:?}", function_name, other);
            }
        };
        
        Ok(Self { 
            recursive,
            timeout,
        })
    }
}

#[derive(Clone, Copy)]
struct EventCategory {
    kind: EventKind,
}
impl EventCategory {
    fn new(kind: EventKind) -> Self {
        Self {
            kind,
        }
    }
    fn category(self) -> &'static str {
        match self.kind {
            EventKind::Access(access) => match access {
                AccessKind::Read => "Read",
                AccessKind::Open(AccessMode::Execute) => "Execute",
                // these are noisy so just put them under access
                AccessKind::Open(AccessMode::Any) => "Access",
                AccessKind::Open(_) => "Open",
                AccessKind::Close(_) => "Close",
                _ => "Access",
            },
            EventKind::Create(_) => "Create",
            EventKind::Modify(modify) => match modify {
                ModifyKind::Data(_) => "Modify::Data",
                ModifyKind::Metadata(_) => "Modify::Metadata",
                ModifyKind::Name(_) => "Rename",
                _ => "Modify::Other",
            },
            EventKind::Remove(_) => "Remove",
            EventKind::Other => "Other",
            EventKind::Any => "Unknown",
        }
    }
    fn stringify_kind(self) -> &'static str {
        match self.kind {
            EventKind::Access(access) => match access {
                AccessKind::Read => "Read",
                AccessKind::Open(mode) => match mode {
                    AccessMode::Execute => "Open::Execute",
                    AccessMode::Read => "Open::Read",
                    AccessMode::Write => "Open::Write",
                    AccessMode::Other => "Open::Other",
                    AccessMode::Any => "Open::Any",
                },
                AccessKind::Close(mode) => match mode {
                    AccessMode::Execute => "Close::Execute",
                    AccessMode::Read => "Close::Read",
                    AccessMode::Write => "Close::Write",
                    AccessMode::Other => "Close::Other",
                    AccessMode::Any => "Close::Any",
                },
                AccessKind::Any => "Access::Any",
                AccessKind::Other => "Access::Other",
            },
            EventKind::Create(create) => match create {
                CreateKind::File => "Create::File",
                CreateKind::Folder => "Create::Directory",
                CreateKind::Other => "Create::Other",
                CreateKind::Any => "Create::Any",
            },
            EventKind::Modify(modify) => match modify {
                ModifyKind::Name(rename) => match rename {
                    RenameMode::Any => "Rename::Any",
                    RenameMode::From => "Rename::From",
                    RenameMode::To => "Rename::To",
                    RenameMode::Both => "Rename::Both",
                    RenameMode::Other => "Rename::Other",
                },
                ModifyKind::Data(data) => match data {
                    DataChange::Any => "Modify::Data",
                    DataChange::Content => "Modify::Data::Content",
                    DataChange::Size => "Modify::Data::Size",
                    DataChange::Other => "Modify::Data::Other",
                },
                ModifyKind::Metadata(meta) => match meta {
                    MetadataKind::AccessTime => "Modify::Metadata::AccessTime",
                    MetadataKind::WriteTime => "Modify::Metadata::WriteTime",
                    MetadataKind::Ownership => "Modify::Metadata::Ownership",
                    MetadataKind::Permissions => "Modify::Metadata::Permissions",
                    MetadataKind::Extended => "Modify::Metadata::Extended",
                    MetadataKind::Other => "Modify::Metadata::Other",
                    MetadataKind::Any => "Modify::Metadata::Any",
                },
                ModifyKind::Any => "Modify::Any",
                ModifyKind::Other => "Modify::Other",
            },
            EventKind::Remove(remove) => match remove {
                RemoveKind::File => "Remove::File",
                RemoveKind::Folder => "Remove::Directory",
                RemoveKind::Other => "Remove::Other",
                RemoveKind::Any => "Remove::Any",
            },
            EventKind::Other => "Other",
            EventKind::Any => "Unknown",
        }
    }
}

fn create_event_table(event: Event, event_category: EventCategory, luau: &Lua) -> LuaResult<LuaTable> {
    let paths_table = luau.create_table_with_capacity(event.paths.len(), 0)?;
    for path in event.paths.iter() {
        let s = normalize_path(path.to_string_lossy().as_ref());
        paths_table.raw_push(luau.create_string(s)?)?;
    }  
    TableBuilder::create(luau)?
        .with_value("paths", paths_table)?
        .with_value("kind", ok_string(event_category.stringify_kind(), luau)?)?
        .with_value("is_write", {
            matches!(event_category.stringify_kind(), "Create::File" | "Close::Write" | "Modify::Data")
        })?
        .build_readonly()
}

pub fn watch<P: AsRef<Path>>(luau: &Lua, paths: Vec<P>, options: WatchOptions, function_name: &'static str) -> LuaValueResult {
    let (tx, rx) = crossbeam_channel::unbounded::<Event>();
    
    let mut watcher = match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) =>  {
                tx.send(event).unwrap_or_else(|err| {
                    eprintln!("Unable to send event due to {}", err);
                });
            },
            Err(err) => {
                eprintln!("Unable to send message due to {}", err);
            }
        };
    }) {
        Ok(watcher) => watcher,
        Err(err) => {
            return wrap_err!("{} unable to create 'notify' filesystem watcher due to err: {}", function_name, err);
        }
    };
    let recursive = if options.recursive { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };
    for path in paths {
        if let Err(err) = watcher.watch(path.as_ref(), recursive) {
            return wrap_err!("{} unable to watch path '{}' due to err: {}", function_name, path.as_ref().display(), err);
        }
    }

    let arc_rx = Arc::new(Mutex::new(rx));
    let watcher = Arc::new(watcher);

    ok_function_mut({
        let arc_rx = Arc::clone(&arc_rx);
        move | luau: &Lua, _value: LuaValue | -> LuaMultiResult {
            // need to clone watcher here just to keep it alive (so it doesn't disconnect while we're iterating)
            let _watcher = Arc::clone(&watcher);
            let rx = match arc_rx.try_lock() {
                Ok(rx) => rx,
                Err(err) => {
                    panic!("{} unexpectedly cannot lock the crossbeam event receiver due to err: {}", function_name, err);
                }
            };
            match rx.recv_timeout(options.timeout) {
                // if an event is received we return its category and an event info table describing
                // what specific kind of event was received and what paths were accessed/modified/written to/etc.
                Ok(event) => {
                    let event_category = EventCategory::new(event.kind);
                    let event_table = ok_table(create_event_table(event, event_category, luau))?;
                    let category_str = ok_string(event_category.category(), luau)?;
                    Ok(LuaMultiValue::from_vec(vec![category_str, event_table]))
                },
                // if no event recv by timeout we return "Timeout", { kind = "None", paths = {} }
                // so we don't indefinitely block the luau vm until the next event recv
                Err(RecvTimeoutError::Timeout) => {
                    Ok(LuaMultiValue::from_vec(vec![
                        ok_string("None", luau)?,
                        ok_table(
                            TableBuilder::create(luau)?
                                .with_value("paths", luau.create_table()?)?
                                .with_value("kind", "None::Timeout")?
                                .with_value("is_write", false)?
                                .build()
                        )?
                    ]))
                },
                // the channel has somehow gotten disconnected, this means either the sender panicked or smth
                // or we somehow dropped the watcher
                // - either case probably means there's a bug in seal or notify or crossbeam
                // - if we just returned nil here to stop iteration, users would wonder why their for loop stopped iterating
                // - if we wrap_err! here, users would pcall this and we wouldn't know that users are actually getting this
                // - so we panic so users may report this and we can investigate
                Err(RecvTimeoutError::Disconnected) => {
                    // Ok(LuaMultiValue::from_vec(vec![LuaNil]))
                    panic!(
                        "{}: {}\n{}\n{}\n{}\n{}",
                        "filesystem watcher channel disconnected unexpectedly",
                        "This closure owns arc_rx and should not lose its sender unless:",
                        "  - the watcher was dropped prematurely,",
                        "  - the notify callback panicked",
                        "  - there's a bug in seal, crossbeam, or notify itself.",
                        "Please report this with reproduction steps if possible.",
                    );
                }
            }
        }
    }, luau)
}