use std::sync::mpsc;
use serenity::prelude::TypeMapKey;

pub struct Packet {
	pub kind: PacketKind,
	pub content: String,
}

impl Packet {
	pub fn new(kind : PacketKind, content: String) -> Self {
		Self {
			kind,
			content,
		}
	}
}


pub enum PacketKind {
	GuildName,
}

pub struct Sender;

impl TypeMapKey for Sender {
	type Value = mpsc::Sender<Packet>;
}