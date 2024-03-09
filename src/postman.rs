use std::sync::mpsc;
use serenity::prelude::TypeMapKey;

pub struct Message {
	kind: MessageType,
	content: String,
}

impl Message {
	pub fn new(kind : MessageType, content: String) -> Self {
		Self {
			kind,
			content,
		}
	}
}


pub enum MessageType {
	GuildName,
}

pub struct Sender;

impl TypeMapKey for Sender {
    type Value = mpsc::Sender<Message>;
}