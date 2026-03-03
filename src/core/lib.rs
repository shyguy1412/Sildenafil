mod events;
mod journal;

use std::{
    io::Read,
    sync::{OnceLock, RwLock},
    thread::JoinHandle,
};

use bondage::*;
use neon::prelude::*;

use crate::{events::Event, journal::Journal};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const EVENT_POLL_RATE: (u64, u64) = (60, 1000); //60 times per second in millis
const EVENT_POLL_INTERVAL: u64 = EVENT_POLL_RATE.1 / EVENT_POLL_RATE.0;

static EVENT_CALLBACK: RwLock<Option<Root<JsFunction>>> = RwLock::new(None);

fn get_linux_graphics_config() -> Result<String> {
    const PATH: &str = "/home/shy/.local/share/Steam/steamapps/common/Elite Dangerous/Products/elite-dangerous-odyssey-64/GraphicsConfiguration.xml";
    let mut contents = String::new();
    let _ = std::fs::File::open(PATH)?.read_to_string(&mut contents);
    Ok(contents)
}

#[export]
fn get_graphics_config(ctx: &mut Cx<'_>) -> NeonResult<String> {
    let conf = get_linux_graphics_config().map_err(|err| ctx.throw_error(err.to_string()).unwrap());

    conf
}

static EVENT_THREAD: OnceLock<JoinHandle<()>> = OnceLock::new();

#[main]
fn main(_: ModuleContext) -> NeonResult<()> {
    let event_thread = std::thread::spawn(move || {
        std::thread::park();
        let journal = &mut Journal::new();
        loop {
            let event = journal.next();
            match event {
                Some(event) => dispatch_event(event),
                None => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        }
    });

    let _ = EVENT_THREAD.set(event_thread);

    Ok(())
}

#[export]
fn resume(_: &mut Cx<'_>) -> NeonResult<()> {
    Ok(EVENT_THREAD
        .get()
        .map(|t| t.thread().unpark())
        .unwrap_or(()))
}

// async fn event_loop(events_location: &String) -> Option<()> {
//     let mut clock = smol::Timer::interval(std::time::Duration::from_millis(EVENT_POLL_INTERVAL));

//     let file = &mut open_next_journal(&events_location).await?.lines();

//     loop {
//         let line = file.next().await;
//         let Some(Ok(line)) = line else {
//             clock.next().await;
//             continue;
//         };

//         let event: Event = match serde_json::from_str(&line) {
//             Ok(event) => event,
//             Err(error) => {
//                 console_log(format!("{error:?}: {line}"));
//                 continue;
//             }
//         };
//         console_log(event.name());

//         if let events::Event::Shutdown(..) = event {
//             return None;
//         }
//     }
// }

#[export(cb = "<T extends keyof EventVariants>(event:T, data:Event<T>) => void")]
pub fn set_event_listener(_: &mut Cx<'_>, cb: Root<JsFunction>) -> NeonResult<()> {
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
