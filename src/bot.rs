use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

mod token;

const HELP_MESSAGE: &str = "Hello there, Human! I am a messenger for the wandering penwing.";
const HELP_COMMAND: &str = "!jiji";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == HELP_COMMAND {
            if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

pub async fn start_discord_bot() {
    println!("Creation process started");
    let maybe_client = Client::builder(token::TOKEN)
        .event_handler(Handler)
        .await
        .map_err(|why| format!("Client error: {:?}", why));

    println!("Got a result");

    if let Ok(mut client) = maybe_client {
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
            return;
        }
    } else {
        println!("No Client");
        return;
    }
}
