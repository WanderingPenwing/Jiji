use std::sync::mpsc;
use serenity::prelude::TypeMapKey;
use crate::discord_structure;
use std::sync::Mutex;

pub enum Packet {
	Guild(discord_structure::Guild),
	Channel(discord_structure::Channel),
	Message(discord_structure::Message),
	SendMessage(String, String),
	ChannelEnd(String, String),
	FetchChannels(String),
	FetchMessages(String, String, String),
	FinishedRequest,
	Error(String),
}

pub struct Sender;

impl TypeMapKey for Sender {
	type Value = mpsc::Sender<Packet>;
}

pub struct Receiver;

impl TypeMapKey for Receiver {
	type Value = Mutex<mpsc::Receiver<Packet>>;
}