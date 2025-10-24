mod events;

use std::vec;

use bondage::*;

const EVENT_POLL_RATE: (u64, u64) = (1, 10000); //60 times per second in millis
const EVENT_POLL_INTERVAL: u64 = EVENT_POLL_RATE.1 / EVENT_POLL_RATE.0;

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
        EVENT_SYSTEM.dispatch_event(TestEvent {
            foo: "its staring".to_string(),
        });
        runtime.block_on(event_loop(events_location));
        EVENT_SYSTEM.dispatch_event(TestEvent {
            foo: "its joever".to_string(),
        });
    });
}

async fn event_loop(events_location: String) {
    let mut clock = tokio::time::interval(std::time::Duration::from_millis(EVENT_POLL_INTERVAL));

    let json = r#"{ "timestamp":"2025-10-19T17:56:57Z", "event":"CommunityGoal", "CurrentGoals":[ { "CGID":833, "Title":"Brewer Corporation Strategic Order", "SystemName":"HIP 90578", "MarketName":"Trailblazer Dream", "Expiry":"2025-10-28T14:00:00Z", "IsComplete":false, "CurrentTotal":134830429, "PlayerContribution":0, "NumContributors":14167, "TopTier":{ "Name":"Tier 5", "Bonus":"" }, "TopRankSize":10, "PlayerInTopRank":false, "TierReached":"Tier 3", "PlayerPercentileBand":100, "Bonus":45000000 } ] }"#;

    let cg: events::CommunityGoal = serde_json::from_str(json).expect("Guranteed by argument");

    
    loop {
        clock.tick().await;
        //detect and dispatch all events
        EVENT_SYSTEM.dispatch_event(cg.clone());

        //read dir
        //pick latest log
        //read log
        //parse events
    }
}

#[derive(Event, Transferable)]
struct TestEvent {
    foo: String,
}

#[derive(Transferable)]
struct Foo {
    bar: Option<Vec<f64>>,
}
