use serenity::{
	async_trait,
	model::{channel::Message, gateway::Ready},
	prelude::*,
};
use serenity::model::prelude::GuildId;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread;
use std::sync::Arc;

use crate::postman;
use crate::discord_structure;

mod token;

const HELP_MESSAGE: &str = "Hello there, Human! I am a messenger for the wandering penwing.";
const HELP_COMMAND: &str = "!jiji";
const PACKET_REFRESH : u64 = 500;

struct Handler {
	is_loop_running: AtomicBool,
}


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
		println!("bot : {} is connected!", ready.user.name);
	}
	
	async fn cache_ready(&self, context: Context, _guilds: Vec<GuildId>) {
		println!("bot : cache built successfully!");
		
		let context = Arc::new(context);
		
		if !self.is_loop_running.load(Ordering::Relaxed) {
            // We have to clone the Arc, as it gets moved into the new thread.
            let context1 = Arc::clone(&context);
            // tokio::spawn creates a new green thread that can run in parallel with the rest of
            // the application.
            tokio::spawn(async move {
                get_guilds(&context1).await;
            });

			let context2 = Arc::clone(&context);
            tokio::spawn(async move {
                loop {
                    check_packets(&context2).await;
                    thread::sleep(Duration::from_millis(PACKET_REFRESH));
                }
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
	}
}

async fn check_packets(context: &Context) {
	if let Some(receiver_mutex) = context.data.read().await.get::<postman::Receiver>() {
		if let Ok(receiver) = receiver_mutex.lock() {
			while let Ok(packet) = receiver.try_recv() {
				match packet {
					postman::Packet::FetchChannels(guild_id) => {
						println!("bot : fetch channels request received : '{}'", guild_id);
					}
					_ => {
						println!("bot : unhandled packet");
					}
				}
			}
		} else {
			println!("bot : failed to lock receiver");
		}
	} else {
		println!("bot : failed to retrieve receiver");
	}
}

async fn get_guilds(context: &Context) {
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

pub async fn start_discord_bot(sender: mpsc::Sender<postman::Packet>, receiver: Mutex<mpsc::Receiver<postman::Packet>>) {
	println!("bot : connection process started...");
	let maybe_client = Client::builder(token::TOKEN)
		.event_handler(Handler {
			is_loop_running: AtomicBool::new(false),
		})
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
