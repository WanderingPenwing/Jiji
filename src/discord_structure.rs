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
			id,
			guild_id,
			messages: vec![],
		}
	}
	
	pub fn greetings(&mut self) {
		self.messages.push(Message::new("-".into(), "0".into(), self.id.clone(), self.guild_id.clone(), "start of the conversation".into()));
	}
}

#[derive(PartialEq, Clone)]
pub struct Message {
	pub author_name: String,
	pub id: String,
	pub channel_id: String,
	pub guild_id: String,
	pub content: String,
}

impl Message {
	pub fn new(author_name: String, id: String, channel_id: String, guild_id: String, content: String) -> Self {
		Self {
			author_name,
			id,
			channel_id,
			guild_id,
			content,
		}
	}
}