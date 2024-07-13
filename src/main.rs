use eframe::egui;
use image::GenericImageView;
use std::{error::Error, sync::Arc, sync::mpsc, thread, time};
use tokio::runtime::Runtime;
use std::sync::Mutex;

mod bot;
mod postman;
mod discord_structure;

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
	time_watch: f32,
	pending_bot_requests: usize,
	current_message: String,
	channels_to_notify: Vec<String>,
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
			time_watch: 0.0,
			pending_bot_requests: 0,
			current_message: "".into(),
			channels_to_notify: vec![],
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
						self.guilds[i].add_channel(channel.clone());
					}
				}
				postman::Packet::Message(message) => {
					println!("gui : message received : '{}'", message.content);
					
					let mut guild: Option<usize> = None;
					
					for i in 0..self.guilds.len() {
						if self.guilds[i].id != message.guild_id {
							continue
						}
						guild = Some(i);
					}
					
					
					if let Some(guild_index) = guild {
						let mut unkown_channel = true;
						for i in 0..self.guilds[guild_index].channels.len() {
							if self.guilds[guild_index].channels[i].id != message.channel_id {
								continue
							}
							self.guilds[guild_index].channels[i].insert(message.clone());
							unkown_channel = false;
							println!("gui : message put in : '{}'", self.guilds[guild_index].channels[i].name);
						}
						
						if unkown_channel {
							println!("gui : unkown channel");
							self.guilds[guild_index].channels.push(discord_structure::Channel::create(message.channel_id.clone(), message.channel_id.clone(), message.guild_id.clone()));
							let last = self.guilds[guild_index].channels.len() - 1;
							self.guilds[guild_index].channels[last].insert(message.clone());
						}
					} else {
						println!("gui : message guild issue : '{}'", message.guild_id);
						
						println!("gui : guilds {:?}", self.guilds.clone().into_iter().map(|guild| { guild.id.clone()}).collect::<Vec<String>>());
					}
				}
				postman::Packet::ChannelEnd(guild_id, channel_id) => {
					println!("gui : end of channel : '{}'", channel_id);
					
					let mut guild: Option<usize> = None;
					
					for i in 0..self.guilds.len() {
						if self.guilds[i].id != guild_id {
							continue
						}
						guild = Some(i);
					}
					
					if let Some(guild_index) = guild {
						for i in 0..self.guilds[guild_index].channels.len() {
							if self.guilds[guild_index].channels[i].id != channel_id {
								continue
							}
							self.guilds[guild_index].channels[i].end();
						}
					}
					
				}
				postman::Packet::Error(reason) => {
					println!("gui : error received {}", reason);
				}
				postman::Packet::FinishedRequest => {
					self.pending_bot_requests = self.pending_bot_requests.checked_sub(1).unwrap_or(0);
				}
				_ => {
					println!("gui : unhandled packet");
				}
			}
		}
		
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
		//self.runtime.shutdown_background();
	}
}

impl Jiji {
	pub fn draw_selection(&mut self, ctx: &egui::Context) {
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
								self.selected_channel = None;
							}
							for i in 0..self.guilds.len() {
								if ui.add(egui::SelectableLabel::new(self.selected_guild == Some(i), self.guilds[i].name.clone())).clicked() {
									self.selected_guild = Some(i);
									self.selected_channel = None;
									
									if self.guilds[i].channels.len() == 0 && self.guilds[i].id != "dm" {
										let _ = self.sender.send(postman::Packet::FetchChannels(self.guilds[i].id.clone()));
										
										self.pending_bot_requests += 1;
									}
									
								}
							}
						});
						
					
					//let _ = self.sender.send(postman::Packet::FetchChannels(self.guilds[*selected_guild_index].id.clone()));
					if let Some(selected_guild_index) = &self.selected_guild {
						if self.guilds[*selected_guild_index].channels.len() != 0 {
							let selected_channel_text = if let Some(selected_channel_index) = &self.selected_channel {
								self.guilds[*selected_guild_index].channels[*selected_channel_index].display()
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
										if ui.add(egui::SelectableLabel::new(self.selected_channel == Some(i), self.guilds[*selected_guild_index].channels[i].display())).clicked() {
											self.selected_channel = Some(i);
											if self.guilds[*selected_guild_index].channels[i].messages.len() == 1 {
												let _ = self.sender.send(postman::Packet::FetchMessages(self.guilds[*selected_guild_index].id.clone(), self.guilds[*selected_guild_index].channels[i].id.clone(), "".into()));
												
												self.pending_bot_requests += 1;
											}
										}
									}
								});
							
							if let Some(selected_channel_index) = &self.selected_channel {
								ui.checkbox(&mut self.guilds[*selected_guild_index].channels[*selected_channel_index].notify, "notify");
							}
						}
					}
				});
			});
		
	}
	
	pub fn draw_infobar(&mut self, ctx: &egui::Context) {
		egui::TopBottomPanel::bottom("infobar")
			.resizable(false)
			.show(ctx, |ui| {
				if let Some(guild_index) = self.selected_guild {
					if let Some(channel_index) = self.selected_channel {
						ui.label("");
						ui.horizontal(|ui| {
							if ui.button(">").clicked() {
								let _ = self.sender.send(postman::Packet::SendMessage(self.guilds[guild_index].channels[channel_index].id.clone(), self.current_message.clone()));
								self.current_message = "".to_string();
							}
							egui::ScrollArea::vertical()
								.show(ui, |ui| {
									let _response = ui.add(egui::TextEdit::multiline(&mut self.current_message)
										.desired_width(f32::INFINITY)
										.lock_focus(true));
								});
						});
					}
				}
				ui.horizontal(|ui| {
					ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
						ui.label(&format!("time per frame : {:.1} ms", self.time_watch));
						
						ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
							if self.pending_bot_requests > 0 {
								ui.label(&format!(" {} pending", self.pending_bot_requests));
							}
						});
					});
				});
			});
	}
	
	pub fn draw_feed(&mut self, ctx: &egui::Context) {
		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical()
					.stick_to_bottom(true)
					.show(ui, |ui| {
				if let Some(selected_guild_index) = &self.selected_guild {
					if let Some(selected_channel_index) = &self.selected_channel {
						self.guilds[*selected_guild_index].channels[*selected_channel_index].unread = false;
						
						let mut last_author = "".to_string();
						
						if self.guilds[*selected_guild_index].channels[*selected_channel_index].messages.len() < 2 {
							return
						}
						
						for message in &self.guilds[*selected_guild_index].channels[*selected_channel_index].messages {							
							if message.author_name == "+" {
								if ui.button("+").clicked() {
									if let Some(selected_guild_index) = &self.selected_guild {
										if let Some(selected_channel_index) = &self.selected_channel {
											let _ = self.sender.send(postman::Packet::FetchMessages(
												self.guilds[*selected_guild_index].id.clone(), 
												self.guilds[*selected_guild_index].channels[*selected_channel_index].id.clone(), 
												self.guilds[*selected_guild_index].channels[*selected_channel_index].messages[1].id.clone(),
											));
											self.pending_bot_requests += 1;
										}
									}
								}
								continue
							}
							if message.author_name != last_author {
								ui.separator();
								ui.colored_label(hex_str_to_color("#3399ff"), &message.author_name);
							} else {
								ui.label("");
							}
							ui.label(&message.content);
							last_author = message.author_name.clone();
						}
					}
				}
			});
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

pub fn hex_str_to_color(hex_str: &str) -> egui::Color32 {
	egui::Color32::from_hex(hex_str).unwrap_or_else(|_| egui::Color32::WHITE)
}