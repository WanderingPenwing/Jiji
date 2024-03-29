use eframe::egui;
use image::GenericImageView;
use std::{error::Error, sync::Arc, sync::mpsc, thread, time};
use tokio::runtime::Runtime;
use std::sync::Mutex;

mod bot;
mod postman;
mod discord_structure;

const MAX_FPS: f32 = 30.0;

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
	let icon_data = load_icon().unwrap_or_default();

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
	guilds: Vec<discord_structure::Guild>,
	selected_guild: Option<discord_structure::Guild>,
}

impl Jiji {
	fn new(sender: mpsc::Sender<postman::Packet>, receiver: mpsc::Receiver<postman::Packet>) -> Self {
		Self {
			next_frame: time::Instant::now(),
			sender,
			receiver,
			guilds: vec![],
			selected_guild: None,
		}
	}
}

impl eframe::App for Jiji {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		thread::sleep(time::Duration::from_secs_f32(
			((1.0 / MAX_FPS) - self.next_frame.elapsed().as_secs_f32()).max(0.0),
		));
		self.next_frame = time::Instant::now();
		
		while let Ok(packet) = self.receiver.try_recv() {
			match packet {
				postman::Packet::Guild(guild) => {
					println!("gui : guild received : '{}'", guild.name);
					self.guilds.push(guild);
				}
				postman::Packet::Channel(channel) => {
					println!("gui : channel received : '{}'", channel.name);
				}
				postman::Packet::Message(message) => {
					println!("gui : message received : '{}'", message.content);
				}
				_ => {
					println!("unhandled packet");
				}
			}
		}

		self.draw_feed(ctx);
	}

	fn on_exit(&mut self, _gl: std::option::Option<&eframe::glow::Context>) {
		//self.runtime.shutdown_background();
	}
}

impl Jiji {
	pub fn draw_feed(&mut self, ctx: &egui::Context) {
		egui::TopBottomPanel::top("server_selection")
			.resizable(false)
			.show(ctx, |ui| {
				ui.horizontal(|ui| {
					ui.label("Where do you want to look ? ");
					let selected_text = if let Some(guild) = &self.selected_guild {
						guild.name.clone()
					} else {
						"None".to_string()
					};
					
					egui::ComboBox::from_label("")
						.selected_text(format!("{}", selected_text))
						.show_ui(ui, |ui| {
							ui.style_mut().wrap = Some(false);
							ui.set_min_width(60.0);
							if ui.add(egui::SelectableLabel::new(self.selected_guild == None, "None")).clicked() {
							    self.selected_guild = None;
							}
							for guild in self.guilds.clone() {
								if ui.add(egui::SelectableLabel::new(self.selected_guild == Some(guild.clone()), guild.name.clone())).clicked() {
								    self.selected_guild = Some(guild);
								}
							}
						});
					
					if let Some(guild) = &self.selected_guild {
						if guild.channels.len() == 0 {
							if ui.add(egui::Button::new("get channels")).clicked() {
								let _ = self.sender.send(postman::Packet::FetchChannels(guild.id.clone()));
							}
						}
					}
				});
			});
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.label("General Kenobi");
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