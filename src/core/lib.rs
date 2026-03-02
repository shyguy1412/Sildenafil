mod events;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, RwLock},
    thread::JoinHandle,
};

use bondage::*;
use neon::prelude::*;
use smol::{io::AsyncBufReadExt, stream::StreamExt};

use crate::events::Event;

const EVENT_POLL_RATE: (u64, u64) = (60, 1000); //60 times per second in millis
const EVENT_POLL_INTERVAL: u64 = EVENT_POLL_RATE.1 / EVENT_POLL_RATE.0;

static EVENT_CALLBACK: RwLock<Option<Root<JsFunction>>> = RwLock::new(None);

fn get_windows_events_location(ctx: &mut ModuleContext) -> NeonResult<String> {
    //!This doesnt check for a steam lib on a different drive

    let user_profile = std::env::var("USERPROFILE").map_err(|_| {
        ctx.throw_error::<&str, std::convert::Infallible>("Can not access %USERPROFILE%")
            .unwrap_err()
    })?;

    Ok(format!(
        "{}\\Saved Games\\Frontier Developments\\Elite Dangerous",
        user_profile
    ))
}

fn get_linux_events_location(ctx: &mut ModuleContext) -> NeonResult<String> {
    //!This doesnt check for a steam lib on a different drive

    let user_home = std::env::var("HOME").map_err(|_| {
        ctx.throw_error::<&str, std::convert::Infallible>("Can not access $HOME")
            .unwrap_err()
    })?;

    Ok(format!(
        "{}/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous",
        user_home
    ))
}

static EVENT_THREAD: OnceLock<JoinHandle<()>> = OnceLock::new();

#[main]
fn main(mut ctx: ModuleContext) -> NeonResult<()> {
    let events_location = match std::env::consts::OS {
        "windows" => get_windows_events_location(&mut ctx)?,
        "linux" => get_linux_events_location(&mut ctx)?,
        os => return ctx.throw_error(format!("`{}` is currently not supported", os))?,
    };

    let event_thread = std::thread::spawn(move || {
        loop {
            smol::block_on(event_loop(&events_location));
            console_log("Restart Loop");
        }
    });

    let _ = EVENT_THREAD.set(event_thread);

    Ok(())
}

#[export]
fn resume() -> NeonResult<()> {
    Ok(EVENT_THREAD
        .get()
        .map(|t| t.thread().unpark())
        .unwrap_or(()))
}

static KNOWN_JOURNALS: Mutex<Vec<PathBuf>> = Mutex::new(vec![]);

async fn open_current_journal(
    events_location: &String,
) -> Option<smol::io::BufReader<smol::fs::File>> {
    let current_journal = get_current_journal(&events_location)?;

    let mut known_journals_lock = match KNOWN_JOURNALS.lock() {
        Ok(lock) => lock,
        Err(err) => err.into_inner(),
    };

    let is_known = known_journals_lock
        .iter()
        .any(|known_buff| *known_buff == current_journal);

    if is_known {
        console_log("Waiting for new journal");
        std::thread::park();
        return None;
    };

    let file = smol::fs::File::open(&current_journal)
        .await
        .ok()
        .map(smol::io::BufReader::new);

    known_journals_lock.push(current_journal);

    file
}

async fn event_loop(events_location: &String) -> Option<()> {
    let mut clock = smol::Timer::interval(std::time::Duration::from_millis(EVENT_POLL_INTERVAL));

    let file = &mut open_current_journal(&events_location).await?.lines();

    loop {
        let line = file.next().await;
        let Some(Ok(line)) = line else {
            clock.next().await;
            continue;
        };

        let event: Event = match serde_json::from_str(&line) {
            Ok(event) => event,
            Err(error) => {
                console_log(format!("{error:?}: {line}"));
                continue;
            }
        };
        console_log(event.name());

        if let events::Event::Shutdown(..) = event {
            return None;
        }
    }
}

fn get_current_journal(path: &String) -> Option<PathBuf> {
    let Ok(files) = std::fs::read_dir(path) else {
        return None;
    };

    let mut files: Vec<_> = files
        .filter_map(|file| file.ok())
        .filter_map(|file| file.file_name().into_string().ok())
        .map(|file| (file.replace(non_numeric, ""), file))
        .filter_map(|(date, file)| u64::from_str_radix(&date, 10).ok().map(|date| (file, date)))
        .collect();

    files.sort_by(|a, b| a.1.cmp(&b.1));

    let current_journal = files.pop().map(|(f, _)| Path::new(path).join(f));

    current_journal
}

fn non_numeric(char: char) -> bool {
    !char.is_numeric()
}

#[export(cb = "<T extends keyof EventVariants>(event:T, data:Event<T>) => void")]
pub fn set_event_listener(cb: Root<JsFunction>) -> NeonResult<()> {
    let _ = EVENT_CALLBACK.write().map(|mut cell| cell.replace(cb));
    Ok(())
}

#[with_context]
fn dispatch_event(ctx: &mut Cx<'_>, event: Event) -> NeonResult<()> {
    let cb_lock = match EVENT_CALLBACK.read() {
        Ok(lock) => lock,
        Err(error) => error.into_inner(),
    };

    let Some(ref cb) = *cb_lock else {
        return Ok(());
    };

    let event_name = event.name();
    let event_data = event.to_js(ctx);
    let bind = &mut cb.to_inner(ctx).bind(ctx);
    bind.arg(event_name)?;
    bind.arg(event_data)?;
    bind.call::<()>()?;

    Ok(())
}
