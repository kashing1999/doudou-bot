use serenity::client::Context;
use serenity::model::prelude::ChannelId;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn execute_script(script_name: &str, input: &str) -> String {
    let mut cmd = Command::new(script_name);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().expect("failed to spawn command");

    let mut stdin = child
        .stdin
        .take()
        .expect("child did not have a handle to stdin");

    stdin
        .write_all(input.as_bytes())
        .await
        .expect("could not write to stdin");
    drop(stdin); // drop here to signal EOF to stdin

    let out = child.wait_with_output().await.unwrap().stdout;
    String::from_utf8(out).unwrap()
}

pub async fn say_message(ctx: Arc<Context>, message: &str, channel_id: u64) {
    let message = ChannelId(channel_id)
        .send_message(&ctx, |m| m.content(message))
        .await;
    if let Err(why) = message {
        eprintln!("Error sending message: {:?}", why);
    };
}
