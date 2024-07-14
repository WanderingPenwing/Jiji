use eframe::egui;
use std::{sync::Arc, sync::mpsc, thread, time};
use tokio::runtime::Runtime;
use std::sync::Mutex;
use std::path::PathBuf;
use homedir::get_my_home;

mod bot;
mod postman;
mod discord_structure;
mod state;
mod ui;
mod app;

const MAX_FPS: f32 = 30.0;
const RUNNING_REQUEST_REFRESH_DELAY: f32 = 0.2;

fn main() {
	let (bot_tx, gui_rx) = mpsc::channel::<postman::Packet>(); //tx transmiter
	let (gui_tx, bot_rx) = mpsc::channel::<postman::Packet>(); //tx transmiter
	let bot_rx = Mutex::new(bot_rx);
	
	let _handle = thread::spawn(move || {
		println!("main : bot thread spawned");
		let mut rt = Runtime::new().unwrap();
		rt.block_on(bot::start_discord_bot(bot_tx, bot_rx));
	});

	// Run the GUI on the main thread
	gui(gui_tx, gui_rx);
}

fn gui(sender: mpsc::Sender<postman::Packet>, receiver: mpsc::Receiver<postman::Packet>) {
	let icon_data = state::load_icon().unwrap_or_default();

	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([400.0, 300.0])
			.with_icon(Arc::new(icon_data)),
		..Default::default()
	};

	let _ = eframe::run_native("Jiji", options, Box::new(move |_cc| Box::from(Jiji::new(sender, receiver))));
}

struct Jiji {
	next_frame: time::Instant,
	sender: mpsc::Sender<postman::Packet>,
	receiver: mpsc::Receiver<postman::Packet>,
	show_token: bool,
	bot_token: String,
	guilds: Vec<discord_structure::Guild>,
	selected_guild: Option<usize>,
	selected_channel: Option<usize>,
	time_watch: f32,
	pending_bot_requests: usize,
	current_message: String,
	channels_to_notify: Vec<String>,
}

impl Jiji {
	fn new(sender: mpsc::Sender<postman::Packet>, receiver: mpsc::Receiver<postman::Packet>) -> Self {
		let mut app_state = state::load_state(&save_path());
		
		let mut dms = discord_structure::Guild::create("dm".to_string(), "dm".to_string());
		
		for (id, name) in &app_state.dm_channels {
			let mut channel = discord_structure::Channel::create(name.to_string(), id.to_string(), dms.id.clone());
			
			if let Some(index) = app_state.channels_to_notify.iter().position(|x| x == &channel.id) {
				channel.notify = true;
				app_state.channels_to_notify.remove(index);
			}
			
			dms.add_channel(channel);
		}
		
		Self {
			next_frame: time::Instant::now(),
			sender,
			receiver,
			show_token: false,
			bot_token: app_state.bot_token.clone(),
			guilds: vec![dms],
			selected_guild: None,
			selected_channel: None,
			time_watch: 0.0,
			pending_bot_requests: 0,
			current_message: "".into(),
			channels_to_notify: app_state.channels_to_notify.clone(),
		}
	}
}

impl eframe::App for Jiji {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		thread::sleep(time::Duration::from_secs_f32(
			((1.0 / MAX_FPS) - self.next_frame.elapsed().as_secs_f32()).max(0.0),
		));
		self.next_frame = time::Instant::now();
		
		self.handle_packets();
		
		self.draw_selection(ctx);
		
		self.draw_infobar(ctx);

		self.draw_feed(ctx);
		
		self.time_watch = self.next_frame.elapsed().as_micros() as f32 / 1000.0;
		
		if self.pending_bot_requests > 0 && !ctx.input(|i| i.wants_repaint()) {
			thread::sleep(time::Duration::from_secs_f32(RUNNING_REQUEST_REFRESH_DELAY));
			egui::Context::request_repaint(ctx);
		}
	}

	fn on_exit(&mut self, _gl: std::option::Option<&eframe::glow::Context>) {
		self.save_state();
	}
}

pub fn save_path() -> PathBuf {
	get_my_home()
		.unwrap()
		.unwrap()
		.as_path()
		.join(".jiji")
		.join("save.json")
		.to_path_buf()
}
