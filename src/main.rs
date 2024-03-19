use anyhow::Result;
use watchexec::Watchexec;

#[tokio::main]
async fn main() -> Result<()> {
    let wx = Watchexec::new(|action| {
        for event in action.events.iter() {
            eprintln!("EVENT: {event:?}");
        }
        action
    })?;

    wx.config.pathset(["/Volumes/Data exFAT/Temp/src", "."]);

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
