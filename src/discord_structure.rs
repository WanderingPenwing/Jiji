#[derive(PartialEq, Clone)]
pub struct Guild {
	pub name: String,
	pub id: String,
	pub channels: Vec<Channel>,
}

impl Guild {
	pub fn new(name: String, id: String) -> Self {
		Self {
			name,
			id,
			channels: vec![],
		}
	}
}

#[derive(PartialEq, Clone)]
pub struct Channel {
	pub name: String,
	pub id: String,
	pub guild_id: String,
	pub messages: Vec<Message>,
}

impl Channel {
	pub fn new(name: String, id: String, guild_id: String) -> Self {
		Self {
			name,
			id: id.clone(),
			guild_id : guild_id.clone(),
			messages: vec![Message::new("0".into(), id, guild_id, "+".into(), "".into(), "".into())],
		}
	}
	
	pub fn insert(&mut self, message: Message) {
		self.messages.insert(1, message);
		println!("discord_structure : need to compare timestamp");
	}
	
	pub fn end(&mut self) {
		if self.messages[0].author_name != "+" {
			return
		}
		self.messages.remove(0);
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
}

impl Message {
	pub fn new(id: String, channel_id: String, guild_id: String, author_name: String, content: String, timestamp: String) -> Self {
		Self {
			id,
			channel_id,
			guild_id,
			author_name,
			content,
			timestamp,
		}
	}
}