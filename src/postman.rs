use std::sync::mpsc;
use serenity::prelude::TypeMapKey;
use crate::discord_structure;

pub enum Packet {
    Guild(discord_structure::Guild),
    Channel(discord_structure::Channel),
    Message(discord_structure::Message),
}

pub struct Sender;

impl TypeMapKey for Sender {
	type Value = mpsc::Sender<Packet>;
}