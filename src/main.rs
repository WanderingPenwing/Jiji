use eframe::egui;
use image::GenericImageView;
use std::{
	thread, 
	time, 
	sync::Arc,
	error::Error,
	env,
};
use serenity::{
	async_trait,
	model::{channel::Message, gateway::Ready},
	prelude::*,
};

mod token;


const MAX_FPS : f32 = 30.0;
const HELP_MESSAGE: &str = "Hello there, Human! I am a messenger for the wandering penwing.";
const HELP_COMMAND: &str = "!jiji";


//fn main() -> Result<(), eframe::Error> {
//	let icon_data = load_icon().unwrap_or_default();
//	
//	let options = eframe::NativeOptions {
//		viewport: egui::ViewportBuilder::default()
//			.with_inner_size([1200.0, 800.0])
//			.with_icon(Arc::new(icon_data)),
//		..Default::default()
//	};
//
//	eframe::run_native(
//		"Jiji",
//		options,
//		Box::new(move |_cc| Box::from(Jiji::default())),
//	)
//}


struct Jiji {
	next_frame: time::Instant,
}


impl Default for Jiji {
	fn default() -> Self {
		Self {
			next_frame: time::Instant::now(),
		}
	}
}


impl eframe::App for Jiji {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		thread::sleep(time::Duration::from_secs_f32(
			((1.0 / MAX_FPS) - self.next_frame.elapsed().as_secs_f32()).max(0.0),
		));
		self.next_frame = time::Instant::now();

		self.draw_feed(ctx);
	}
}

impl Jiji {
	pub fn draw_feed(&mut self, ctx: &egui::Context) {
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.label("Hello there");
		});
	}
}

pub fn load_icon() -> Result<egui::IconData, Box<dyn Error>> {
	let (icon_rgba, icon_width, icon_height) = {
		let icon = include_bytes!("../assets/icon.png");
		let image = image::load_from_memory(icon)?;
		let rgba = image.clone().into_rgba8().to_vec();
		let (width, height) = image.dimensions();
		(rgba, width, height)
	};

	Ok(egui::IconData {
		rgba: icon_rgba,
		width: icon_width,
		height: icon_height,
	})
}


struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
	if msg.content == HELP_COMMAND {
		if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
			println!("Error sending message: {:?}", why);
	}
	}
}

async fn ready(&self, _: Context, ready: Ready) {
	println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	//let token = env::var(token::TOKEN)
	//.expect("Expected a token in the environment");

	let mut client = Client::new(&token::TOKEN)
	.event_handler(Handler)
	.await
	.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}