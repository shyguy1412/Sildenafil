mod events;

use bondage::*;

const POLL_RATE: (u64, u64) = (1000, 60);
const POLL_INTERVALL: u64 = POLL_RATE.0 / POLL_RATE.1;

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

#[main]
fn main(mut ctx: ModuleContext) {
    let events_location = match std::env::consts::OS {
        "windows" => get_windows_events_location(&mut ctx)?,
        "linux" => get_linux_events_location(&mut ctx)?,
        os => return ctx.throw_error(format!("`{}` is currently not supported", os)), //(ctx.error(format!("`{}` is currently not supported", os))?),
    };

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(event_loop(events_location));
        EVENT_SYSTEM.dispatch_event(TestEvent {
            foo: "its joever".to_string(),
        });
    });
}

async fn event_loop(events_location: String) {
    let mut clock = tokio::time::interval(std::time::Duration::from_millis(POLL_INTERVALL));


    loop {
        clock.tick().await;
        //detect and dispatch all events

        //read dir
        //pick latest log
        //read log
        //parse events 
    }
}

#[derive(Event)]
struct TestEvent {
    foo: String,
}

#[derive(Transferable)]
struct Foo {
    bar: Option<Vec<f64>>,
}
