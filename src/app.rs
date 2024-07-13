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
					println!("gui : guild received : '{}'", guild.name);
					self.guilds.push(guild);
				}
				postman::Packet::Channel(channel) => {
					println!("gui : channel received : '{}'", channel.name);
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
					println!("gui : message received : '{}'", message.content);
					
					let mut guild: Option<usize> = None;
					
					for i in 0..self.guilds.len() {
						if self.guilds[i].id != message.guild_id {
							continue
						}
						guild = Some(i);
					}
					
					
					if let Some(guild_index) = guild {
						let mut unkown_channel = true;
						for i in 0..self.guilds[guild_index].channels.len() {
							if self.guilds[guild_index].channels[i].id != message.channel_id {
								continue
							}
							self.guilds[guild_index].channels[i].insert(message.clone());
							unkown_channel = false;
							println!("gui : message put in : '{}'", self.guilds[guild_index].channels[i].name);
						}
						
						if unkown_channel {
							println!("gui : unkown channel");
							self.guilds[guild_index].channels.push(discord_structure::Channel::create(message.channel_id.clone(), message.channel_id.clone(), message.guild_id.clone()));
							let last = self.guilds[guild_index].channels.len() - 1;
							self.guilds[guild_index].channels[last].insert(message.clone());
						}
					} else {
						println!("gui : message guild issue : '{}'", message.guild_id);
						
						println!("gui : guilds {:?}", self.guilds.clone().into_iter().map(|guild| { guild.id.clone()}).collect::<Vec<String>>());
					}
				}
				postman::Packet::ChannelEnd(guild_id, channel_id) => {
					println!("gui : end of channel : '{}'", channel_id);
					
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
					println!("gui : error received {}", reason);
				}
				postman::Packet::FinishedRequest => {
					self.pending_bot_requests = self.pending_bot_requests.checked_sub(1).unwrap_or(0);
				}
				_ => {
					println!("gui : unhandled packet");
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