use serenity::{
	async_trait,
	model::{channel::Message, gateway::Ready},
	prelude::*,
};
use std::sync::mpsc;
use std::sync::Mutex;
use crate::postman;
use crate::discord_structure;

mod token;

const HELP_MESSAGE: &str = "Hello there, Human! I am a messenger for the wandering penwing.";
const HELP_COMMAND: &str = "!jiji";

struct Handler;


#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
		println!("bot : message received : '{}' from {}", msg.content, msg.author);
		if msg.content == HELP_COMMAND {
			if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
				println!("bot : Error sending message: {:?}", why);
				return
			}
			println!("bot : successfuly sent reply");
		}
	}

	async fn ready(&self, context: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
		let guilds = context.cache.guilds().await;
		
		if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
			for guild_id in guilds {
				let guild_name : String = if let Some(guild) = context.cache.guild(guild_id).await {
					guild.name.clone()
				} else {
					"not found".to_string()
				};
				println!("bot : found guild : '{}' ({})", guild_name.clone(), guild_id.clone());
				
				let guild = discord_structure::Guild::new(guild_name, guild_id.to_string());
				sender.send(postman::Packet::Guild(guild)).expect("Failed to send packet");
			}
		} else {
			println!("bot : failed to retrieve sender");
		}
	}
}

pub async fn start_discord_bot(sender: mpsc::Sender<postman::Packet>, receiver: Mutex<mpsc::Receiver<postman::Packet>>) {
	println!("bot : connection process started...");
	let maybe_client = Client::builder(token::TOKEN)
		.event_handler(Handler)
		.type_map_insert::<postman::Sender>(sender)
		.type_map_insert::<postman::Receiver>(receiver)
		.await
		.map_err(|why| format!("Client error: {:?}", why));
		
	//let mut rx = bot_rx.lock().unwrap(); // Lock the receiver
	//let msg = rx.recv().unwrap(); // Receive a message

	if let Ok(mut client) = maybe_client {
		if let Err(why) = client.start().await {
			println!("bot : client error: {:?}", why);
			return;
		}
	} else {
		println!("bot : no client");
		return;
	}
}
