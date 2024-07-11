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
use std::thread;
use std::sync::Arc;

use crate::postman;
use crate::discord_structure;

mod token;

const HELP_MESSAGE: &str = "Hello there, Human! I am a messenger for the wandering penwing.";
const HELP_COMMAND: &str = "!penwing";
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
				eprintln!("bot : Error sending message: {:?}", why);
				return
			}
			println!("bot : successfuly sent reply");
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

	for packet in packets_received {
		match packet {
			postman::Packet::FetchChannels(guild_id_str) => {
				println!("bot : received FetchChannels packet, for guild '{}'", guild_id_str);
				match guild_id_str.parse::<u64>() {
					Ok(guild_id_u64) => {
						if let Some(guild) = context.cache.guild(guild_id_u64).await {
							match guild.channels(&context.http).await {
								Ok(guild_channels) => {
									if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
										for (channel_id, channel) in guild_channels {
											if channel.kind != ChannelType::Text {
												continue
											}
											let discord_channel = discord_structure::Channel::new(channel.name, format!("{}",channel_id), guild_id_str.to_string());
											sender.send(postman::Packet::Channel(discord_channel)).expect("Failed to send packet");
										}
										println!("bot : sent channels");
										sender.send(postman::Packet::FinishedRequest).expect("Failed to send packet");
									} else {
										println!("bot : failed to retrieve sender");
									}
									
								}
								Err(why) => {
									eprintln!("bot : Failed to get channels : {}", why);
								}
							
							}
						} else {
							println!("bot : guild not found");
						};						
					}
					Err(why) => {
						eprintln!("bot : Failed to parse guild ID string to u64: {}", why);
					}
				}
			}
			postman::Packet::FetchMessages(guild_id_str, channel_id_str, first_message_id_str) => {
				println!("bot : received FetchMessages packet, channel : '{}', first message : {}", channel_id_str, first_message_id_str);
				if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
					match channel_id_str.parse::<u64>() {
						Ok(channel_id_u64) => {
							let channel_id = ChannelId::from(channel_id_u64);

							let maybe_messages = channel_id.messages(&context.http, |retriever| {
						        retriever.after(MessageId::from(158339864557912064)).limit(25)
						    }).await;
							
							match maybe_messages {
								Ok(messages) => {
									println!("bot : got messages");
									
									for message in messages {
										let discord_message = discord_structure::Message::new(message.id.to_string(), channel_id_str.clone(), guild_id_str.clone(), message.author.to_string(), message.content, message.timestamp.to_string());
										sender.send(postman::Packet::Message(discord_message)).expect("Failed to send packet");
									}
								}
								Err(why) => {
									eprintln!("bot : Failed to fetch messages : {}", why);
								}
							
							}
						}
						Err(why) => {
							eprintln!("bot : Failed to parse channel ID string to u64: {}", why);
						}
					}
					
					
					sender.send(postman::Packet::FinishedRequest).expect("Failed to send packet");
				} else {
					println!("bot : failed to retrieve sender");
				}
			}
			_ => {
				println!("bot : unhandled packet");
			}
		}
	}
}


async fn get_guilds(context: &Context) {
	let guilds = context.cache.guilds().await;
		
	if let Some(sender) = context.data.read().await.get::<postman::Sender>() {
		for guild_id in guilds {
			if let Some(guild) = context.cache.guild(guild_id).await {
				println!("bot : found guild '{}'", guild.name.clone());
				let discord_guild = discord_structure::Guild::new(guild.name.clone(), guild_id.to_string());
				sender.send(postman::Packet::Guild(discord_guild)).expect("Failed to send packet");
				
			} else {
				println!("bot : error retrieving guild '{}'", guild_id.clone());
			};
			
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
