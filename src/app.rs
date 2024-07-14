use std::collections::HashMap;

use crate::state;
use crate::save_path;
use crate::postman;
use crate::Jiji;
use crate::discord_structure;


impl Jiji {
	pub fn handle_packets(&mut self) {
		while let Ok(packet) = self.receiver.try_recv() {
			match packet {
				postman::Packet::Guild(guild) => {
					println!("app : guild received : '{}'", guild.name);
					self.guilds.push(guild);
				}
				postman::Packet::Channel(channel) => {
					println!("app : channel received : '{}'", channel.name);
					for i in 0..self.guilds.len() {
						if self.guilds[i].id != channel.guild_id {
							continue
						}
						
						let mut discord_channel = channel.clone(); // finish if with variable clone of channel to update with notify
						
						if let Some(index) = self.channels_to_notify.iter().position(|x| x == &channel.id) {
							discord_channel.notify = true;
							self.channels_to_notify.remove(index);
						}
						
						self.guilds[i].add_channel(discord_channel.clone());
						
							
					}
				}
				postman::Packet::Message(message) => {
					println!("app : message received : '{}'", message.content);
					
					let mut guild: Option<usize> = None;
					
					for i in 0..self.guilds.len() {
						if self.guilds[i].id != message.guild_id {
							continue
						}
						guild = Some(i);
					}
					
					
					if let Some(guild_index) = guild {
						let mut channel: Option<usize> = None;
						
						for i in 0..self.guilds[guild_index].channels.len() {
							if self.guilds[guild_index].channels[i].id != message.channel_id {
								continue
							}
							channel = Some(i);
						}
						
						let channel_index = if let Some(index) = channel {
							index
						} else {
							println!("app: unknown channel");
							self.guilds[guild_index].channels.push(discord_structure::Channel::create(message.channel_id.clone(), message.channel_id.clone(), message.guild_id.clone()));
							self.guilds[guild_index].channels.len() - 1
						};
						
						if self.guilds[guild_index].channels[channel_index].insert(message.clone()) {
							self.guilds[guild_index].unread = true;
						}
						
					} else {
						println!("app : message guild issue : '{}'", message.guild_id);
						
						println!("app : guilds {:?}", self.guilds.clone().into_iter().map(|guild| { guild.id.clone()}).collect::<Vec<String>>());
					}
				}
				postman::Packet::ChannelEnd(guild_id, channel_id) => {
					println!("app : end of channel : '{}'", channel_id);
					
					let mut guild: Option<usize> = None;
					
					for i in 0..self.guilds.len() {
						if self.guilds[i].id != guild_id {
							continue
						}
						guild = Some(i);
					}
					
					if let Some(guild_index) = guild {
						for i in 0..self.guilds[guild_index].channels.len() {
							if self.guilds[guild_index].channels[i].id != channel_id {
								continue
							}
							self.guilds[guild_index].channels[i].end();
						}
					}
					
				}
				postman::Packet::Error(reason) => {
					println!("app : error received {}", reason);
					self.errors.push(reason);
				}
				postman::Packet::FinishedRequest => {
					self.pending_bot_requests = self.pending_bot_requests.checked_sub(1).unwrap_or(0);
				}
				_ => {
					println!("app : unhandled packet");
				}
			}
		}
	}
	
	pub fn save_state(&self) {
		let mut channels_to_notify = self.channels_to_notify.clone();
		let mut dm_channels = HashMap::new();
		
		for g in 0..self.guilds.len() {
			for c in 0..self.guilds[g].channels.len() {
				if self.guilds[g].id == "dm" {
					dm_channels.insert(self.guilds[g].channels[c].id.clone(), self.guilds[g].channels[c].name.clone());
				}
				if !self.guilds[g].channels[c].notify {
					continue
				}
				channels_to_notify.push(self.guilds[g].channels[c].id.clone());
			}
			
		}
		
		let app_state = state::AppState {
			bot_token: self.bot_token.clone(),
			channels_to_notify: channels_to_notify,
			dm_channels: dm_channels,
		};
		let _ = state::save_state(&app_state, save_path().as_path());
	}
}