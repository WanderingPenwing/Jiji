use serenity::{
	async_trait,
	model::{channel::Message, gateway::Ready},
	prelude::*,
};
use serenity::model::prelude::ChannelType;
use serenity::model::id::ChannelId;
use serenity::model::id::MessageId;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::sync::Arc;
use tokio::time::interval;


use crate::postman;
use crate::discord_structure;

pub mod token;

const HELP_MESSAGE: &str = "Hello there, Human! I am a messenger for the wandering penwing.";
const HELP_COMMAND: &str = "!penwing";
const PACKET_REFRESH : u64 = 500;

struct Handler {
	is_loop_running: AtomicBool,
}


#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, context: Context, msg: Message) {
		println!("bot : message received : '{}' from {}", msg.content, msg.author);
		if msg.content == HELP_COMMAND {
			if let Err(why) = msg.channel_id.say(&context.http, HELP_MESSAGE).await {
				eprintln!("bot : Error sending message: {:?}", why);
				return
			}
			println!("bot : successfuly sent reply");
		}
		
		if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
			let author_name = msg.author.name.clone();
			let guild_id = if let Some(id) = msg.guild_id {
				id.to_string()
			} else {
				let private_channel = discord_structure::Channel::create(author_name.clone(), msg.channel_id.to_string(), "dm".to_string());
				sender.send(postman::Packet::Channel(private_channel)).expect("failed to send packet");
				"dm".to_string()
			};
			let discord_message = discord_structure::Message::create(msg.id.to_string(), msg.channel_id.to_string(), guild_id, author_name, msg.content.clone(), msg.timestamp.to_rfc2822()).new();
			sender.send(postman::Packet::Message(discord_message)).expect("failed to send packet");
		} else {
			println!("bot : failed to retrieve sender");
		}
	}

	async fn ready(&self, _context: Context, ready: Ready) {
		println!("bot : {} is connected!", ready.user.name);
	}
	
	async fn cache_ready(&self, context: Context, _guilds: Vec<serenity::model::prelude::GuildId>) {
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
	
			// Use tokio interval instead of spawning a loop
			let context2 = Arc::clone(&context);
			let mut interval = interval(Duration::from_millis(PACKET_REFRESH));
			tokio::spawn(async move {
				loop {
					interval.tick().await;
					check_packets(&context2).await;
				}
			});
	
			// Now that the loop is running, we set the bool to true
			self.is_loop_running.swap(true, Ordering::Relaxed);
		}
	}
}

async fn check_packets(context: &Context) {
	let mut packets_received : Vec<postman::Packet> = vec![];
	
	if let Some(receiver_mutex) = context.data.read().await.get::<postman::Receiver>() {
		if let Ok(receiver) = receiver_mutex.lock() {
			while let Ok(packet) = receiver.try_recv() {
				packets_received.push(packet);
			}
		} else {
			println!("bot : failed to lock receiver");
		}
	} else {
		println!("bot : failed to retrieve receiver");
	}
	
	
	if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
		for packet in packets_received {
			match packet {
				postman::Packet::FetchChannels(guild_id_str) => {
					println!("bot : received FetchChannels packet, for guild '{}'", guild_id_str);
					
					match get_channels(context, guild_id_str).await {
						Ok(_) => {
							println!("bot : successfuly got channels");
						}
						Err(why) => {
							println!("bot : error getting channels : {}", why);
							sender.send(postman::Packet::Error(why)).expect("Failed to send packet");
							sender.send(postman::Packet::FinishedRequest).expect("Failed to send packet");
						}
					}
					
				}
				postman::Packet::FetchMessages(guild_id_str, channel_id_str, first_message_id_str) => {
					println!("bot : received FetchMessages packet, channel : '{}', first message : {}", channel_id_str, first_message_id_str);
					
					match get_messages(context, guild_id_str.clone(), channel_id_str.clone(), first_message_id_str).await {
						Ok(_) => {
							println!("bot : successfuly got messages");
						}
						Err(why) => {
							println!("bot : error getting messages : {}", why);
							sender.send(postman::Packet::Error(why)).expect("Failed to send packet");
							sender.send(postman::Packet::FinishedRequest).expect("Failed to send packet");
						}
					}
				}
				postman::Packet::SendMessage(channel_id_str, content) => {
					println!("bot : received SendMessage packet, channel : '{}', content : {}", channel_id_str, content);
					
					match send_message(context, channel_id_str, content).await {
						Ok(_) => {
							println!("bot : successfuly sent message");
						}
						Err(why) => {
							println!("bot : error sending message : {}", why);
							sender.send(postman::Packet::Error(why)).expect("Failed to send packet");
							sender.send(postman::Packet::FinishedRequest).expect("Failed to send packet");
						}
					}
					
				}
				_ => {
					println!("bot : unhandled packet");
				}
			}
		}
	}
}

async fn send_message(context: &Context, channel_id_str: String, content: String) -> Result<(), String> {
	let channel_id_u64 = channel_id_str.parse::<u64>().map_err(|e| e.to_string())?;
	ChannelId(channel_id_u64).say(&context.http, content).await.map_err(|e| e.to_string())?;
	Ok(())
}

async fn get_channels(context: &Context, guild_id_str: String) -> Result<(), String> {
	if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
		let guild_id_u64 = guild_id_str.parse::<u64>().map_err(|e| e.to_string())?; 
		if let Some(guild) = context.cache.guild(guild_id_u64).await {
			let guild_channels = guild.channels(&context.http).await.map_err(|e| e.to_string())?;
			
			for (channel_id, channel) in guild_channels {
				if channel.kind != ChannelType::Text {
					continue
				}
				let discord_channel = discord_structure::Channel::create(channel.name, format!("{}",channel_id), guild_id_str.to_string());
				sender.send(postman::Packet::Channel(discord_channel)).map_err(|e| e.to_string())?;
			}
			sender.send(postman::Packet::FinishedRequest).map_err(|e| e.to_string())?;
			
		} else {
			return Err("guild not found".to_string())
		}
	} else {
		return Err("failed to retrieve sender".to_string())
	}
	Ok(())				
}

async fn get_messages(context: &Context, guild_id_str: String, channel_id_str: String, first_message_id_str: String) -> Result<(), String> {
	if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
		let channel_id_u64 = channel_id_str.parse::<u64>().map_err(|e| e.to_string())?;
		let channel_id = ChannelId::from(channel_id_u64);

		let messages = if first_message_id_str == "" {
			channel_id.messages(&context.http, |retriever| {
				retriever.limit(25)
			}).await.map_err(|e| e.to_string())?
		} else {
			let first_message_id_u64 = first_message_id_str.parse::<u64>().map_err(|e| e.to_string())?; 
			channel_id.messages(&context.http, |retriever| {
				retriever.before(MessageId::from(first_message_id_u64)).limit(25)
			}).await.map_err(|e| e.to_string())?
		};
				
		for message in &messages {
			let author_name = message.author.name.clone();
			let discord_message = discord_structure::Message::create(message.id.to_string(), channel_id_str.clone(), guild_id_str.clone(), author_name, message.content.clone(), message.timestamp.to_rfc2822());
			sender.send(postman::Packet::Message(discord_message)).map_err(|e| e.to_string())?;
		}
		
		if messages.len() < 25 {
			sender.send(postman::Packet::ChannelEnd(guild_id_str.clone(), channel_id_str.clone())).map_err(|e| e.to_string())?;
		}
		
		sender.send(postman::Packet::FinishedRequest).map_err(|e| e.to_string())?;
	} else {
		return Err("failed to retriev sender".to_string())
	}
	Ok(())
}


async fn get_guilds(context: &Context) {
	let guilds = context.cache.guilds().await;
		
	if let Some(sender) = context.data.read().await.get::<postman::Sender>() {		
		for guild_id in guilds {
			if let Some(guild) = context.cache.guild(guild_id).await {
				println!("bot : found guild '{}'", guild.name.clone());
				let discord_guild = discord_structure::Guild::create(guild.name.clone(), guild_id.to_string());
				sender.send(postman::Packet::Guild(discord_guild)).expect("Failed to send packet");
				
			} else {
				println!("bot : error retrieving guild '{}'", guild_id.clone());
			};
			
		}
	} else {
		println!("bot : failed to retrieve sender");
	}
}

pub async fn start_discord_bot(token: String, sender: mpsc::Sender<postman::Packet>, receiver: Mutex<mpsc::Receiver<postman::Packet>>) {
	println!("bot : connection process started...");
	let maybe_client = Client::builder(&token)
		.event_handler(Handler {
			is_loop_running: AtomicBool::new(false),
		})
		.type_map_insert::<postman::Sender>(sender.clone())
		.type_map_insert::<postman::Receiver>(receiver)
		.await;
	
	match maybe_client {
		Ok(mut client) => {
			if let Err(why) = client.start().await {
				sender.send(postman::Packet::Error(format!("Start error: {:?}", why))).expect("Failed to send packet");
				return;
			}
		}
		Err(why) => {
			sender.send(postman::Packet::Error(format!("Client error: {:?}", why))).expect("Failed to send packet");
		}
	}
}
