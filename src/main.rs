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
	selected_guild: Option<usize>,
	selected_channel: Option<usize>,
}

impl Jiji {
	fn new(sender: mpsc::Sender<postman::Packet>, receiver: mpsc::Receiver<postman::Packet>) -> Self {
		Self {
			next_frame: time::Instant::now(),
			sender,
			receiver,
			guilds: vec![],
			selected_guild: None,
			selected_channel: None,
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
					 for i in 0..self.guilds.len() {
						if self.guilds[i].id != channel.guild_id {
							continue
						}
						self.guilds[i].channels.push(channel.clone());
						println!("gui : channel added to '{}'", self.guilds[i].name);
					}
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
					let selected_guild_text = if let Some(selected_guild_index) = &self.selected_guild {
						self.guilds[*selected_guild_index].name.clone()
					} else {
						"None".to_string()
					};
					
					egui::ComboBox::from_label("")
						.selected_text(format!("{}", selected_guild_text))
						.show_ui(ui, |ui| {
							ui.style_mut().wrap = Some(false);
							ui.set_min_width(60.0);
							if ui.add(egui::SelectableLabel::new(self.selected_guild == None, "None")).clicked() {
								self.selected_guild = None;
							}
							for i in 0..self.guilds.len() {
								if ui.add(egui::SelectableLabel::new(self.selected_guild == Some(i), self.guilds[i].name.clone())).clicked() {
									self.selected_guild = Some(i);
									if self.guilds[i].channels.len() == 0 {
										let _ = self.sender.send(postman::Packet::FetchChannels(self.guilds[i].id.clone()));
									}
								}
							}
						});
						
					
					//let _ = self.sender.send(postman::Packet::FetchChannels(self.guilds[*selected_guild_index].id.clone()));
					if let Some(selected_guild_index) = &self.selected_guild {
						if self.guilds[*selected_guild_index].channels.len() != 0 {
							let selected_channel_text = if let Some(selected_channel_index) = &self.selected_channel {
								self.guilds[*selected_guild_index].channels[*selected_channel_index].name.clone()
							} else {
								"None".to_string()
							};
							
							egui::ComboBox::from_label(":")
								.selected_text(format!("{}", selected_channel_text))
								.show_ui(ui, |ui| {
									ui.style_mut().wrap = Some(false);
									ui.set_min_width(60.0);
									if ui.add(egui::SelectableLabel::new(self.selected_channel == None, "None")).clicked() {
										self.selected_channel = None;
									}
									for i in 0..self.guilds[*selected_guild_index].channels.len() {
										if ui.add(egui::SelectableLabel::new(self.selected_channel == Some(i), self.guilds[*selected_guild_index].channels[i].name.clone())).clicked() {
											self.selected_channel = Some(i);
											if self.guilds[*selected_guild_index].channels[i].messages.len() == 0 {
												let _ = self.sender.send(postman::Packet::FetchMessages(self.guilds[*selected_guild_index].id.clone(), self.guilds[*selected_guild_index].channels[i].id.clone()));
												
												self.guilds[*selected_guild_index].channels[i].greetings();
											}
										}
									}
								});
							
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