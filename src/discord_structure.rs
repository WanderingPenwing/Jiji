use chrono::{DateTime, ParseError};
use notify_rust::Notification;

#[derive(PartialEq, Clone)]
pub struct Guild {
	pub name: String,
	pub id: String,
	pub channels: Vec<Channel>,
	pub unread: bool,
}

impl Guild {
	pub fn create(name: String, id: String) -> Self {
		Self {
			name,
			id,
			channels: vec![],
			unread: false,
		}
	}
	
	pub fn add_channel(&mut self, channel: Channel) {
		let mut already_exist = false;
		
		for i in 0..self.channels.len() {
			if self.channels[i].id != channel.id {
				continue
			}
			already_exist = true;
			
			if self.channels[i].name.parse::<u64>().is_ok() {
				self.channels[i].name = channel.name.clone();
			} else {
				println!("discord_structure : channel already exist but name is not id '{}'",self.channels[i].name);
			}
			
			if channel.notify && !self.channels[i].notify {
				self.channels[i].notify = true;
				if self.channels[i].messages.len() > 0 {
					let _ = Notification::new()
						.summary(&self.channels[i].messages[self.channels[i].messages.len() - 1].author_name)
						.body(&format!("{} - {}", self.name, self.channels[i].name))
						.timeout(0)
						.show();
				}
			}
		}
		
		if !already_exist {
			self.channels.insert(0, channel.clone())
		}
	}
	
	pub fn check_unread(&mut self) {
		self.unread = false;
		
		for channel in &self.channels {
			if channel.unread {
				self.unread = true;
			}
		}
	}
	
	pub fn display(&self) -> String {
		let unread = if self.unread {
			"~ "
		} else {
			""
		};
		format!("{}{}", unread, self.name)
	}
}

#[derive(PartialEq, Clone)]
pub struct Channel {
	pub name: String,
	pub id: String,
	pub guild_id: String,
	pub messages: Vec<Message>,
	pub notify: bool,
	pub unread: bool,
	pub scroll_offset: f32,
	pub content_size: f32,
	pub inner_size: f32,
	pub registered_messages: usize,
}

impl Channel {
	pub fn create(name: String, id: String, guild_id: String) -> Self {
		Self {
			name,
			id: id.clone(),
			guild_id : guild_id.clone(),
			messages: vec![Message::create("0".into(), id, guild_id, "+".into(), "".into(), "".into())],
			notify: false,
			unread: false,
			scroll_offset: 0.0,
			content_size: 0.0,
			inner_size: 0.0,
			registered_messages: 0,
		}
	}
	
	pub fn insert(&mut self, message: Message) -> bool {
		if message.new != "" {
			self.unread = true;
		}
	
		match self.get_index_from_timestamp(&message.timestamp) {
			Ok(index) => {
				self.messages.insert(index, message);
			}
			Err(why) => {
				eprintln!("discord_structure : timestamp error : {}", why);
				self.messages.push(message);
			}
		}
		
		self.unread
	}
	
	pub fn get_index_from_timestamp(&self, message_timestamp: &str) -> Result<usize, ParseError> {
		let new_timestamp = DateTime::parse_from_rfc2822(message_timestamp)?;
		
		let mut index: usize = 0;
				
		for i in 0..self.messages.len() {
			if self.messages[i].timestamp == "" {
				index = i + 1;
				continue
			}
			let current_timestamp = DateTime::parse_from_rfc2822(&self.messages[i].timestamp)?;
			
			if new_timestamp > current_timestamp {
				index = i + 1;
			}
		}
		Ok(index)
	}
	
	pub fn end(&mut self) {
		if self.messages[0].author_name != "+" {
			return
		}
		self.messages.remove(0);
	}
	
	pub fn display(&self) -> String {
		let notify = if self.notify {
			" !"
		} else {
			""
		};
		let unread = if self.unread {
			"~ "
		} else {
			""
		};
		format!("{}{}{}", unread, self.name, notify)
	}
}

#[derive(PartialEq, Clone)]
pub struct Message {
	pub id: String,
	pub channel_id: String,
	pub guild_id: String,
	pub author_name: String,
	pub content: String,
	pub timestamp: String,
	pub new: String,
}

impl Message {
	pub fn create(id: String, channel_id: String, guild_id: String, author_name: String, content: String, timestamp: String) -> Self {
		Self {
			id,
			channel_id,
			guild_id,
			author_name,
			content,
			timestamp,
			new: "".to_string(),
		}
	}
	pub fn new(&mut self) {
		self.new = "yes".to_string();
	}
}