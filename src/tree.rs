
struct Guild {
	name: String,
	id: String,
	channels: Vec<Channel>,
}

struct Channel {
	name: String,
	id: String,
	messages: Vec<Message>,
}

struct Message {
	author: String,
	id: String,
	content: String,
}