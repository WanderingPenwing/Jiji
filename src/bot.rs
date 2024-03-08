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
		println!("Message received : '{}' from {}", msg.content, msg.author);
		if msg.content == HELP_COMMAND {
			println!("Message is command");
			if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
				println!("Error sending message: {:?}", why);
				return
			}
			println!("Successfuly sent message");
		}
	}

	async fn ready(&self, context: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
		let guilds = context.cache.guilds().await;
		
		for guild_id in guilds {
			let guild_name : String = if let Some(guild) = context.cache.guild(guild_id).await {
	            guild.name.clone()
	        } else {
	            "not found".to_string()
	        };
			println!("Guild : '{}' ({})", guild_id, guild_name);
		}
	}
}

pub async fn start_discord_bot() {
	println!("Bot connection process started...");
	let maybe_client = Client::builder(token::TOKEN)
		.event_handler(Handler)
		.await
		.map_err(|why| format!("Client error: {:?}", why));

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
