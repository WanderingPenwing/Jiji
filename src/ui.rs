use eframe::egui;
use chrono::DateTime;

use crate::postman;
use crate::Jiji;

impl Jiji {
	pub fn draw_selection(&mut self, ctx: &egui::Context) {
		egui::TopBottomPanel::top("server_selection")
			.resizable(false)
			.show(ctx, |ui| {
				let mut delete_error: Option<usize> = None;
				for i in 0..self.errors.len() {
					ui.horizontal(|ui| {
						if ui.button("X").clicked() {
							delete_error = Some(i);
						}
						ui.colored_label(hex_str_to_color("#dd5d5a"), &self.errors[i]);
					});
				}
				
				if let Some(index) = delete_error {
					self.errors.remove(index);
				}
				ui.horizontal(|ui| {
					if ui.button("⚙").clicked() {
						self.edit_token = !self.edit_token;
					}
					if self.edit_token {
						ui.label("Token :");
						ui.add(egui::TextEdit::singleline(&mut self.bot_token).desired_width(30.0));
					}
					ui.label("  ");
					let selected_guild_text = if let Some(selected_guild_index) = &self.selected_guild {
						self.guilds[*selected_guild_index].display()
					} else {
						"None".to_string()
					};
					
					egui::ComboBox::from_label("")
						.selected_text(format!("{}", selected_guild_text))
						.show_ui(ui, |ui| {
							ui.style_mut().wrap = Some(false);
							ui.set_min_width(60.0);
							if ui.add(egui::SelectableLabel::new(self.selected_guild == None, "None")).clicked() {
							
								if let Some(selected_guild_index) = &self.selected_guild {
									if let Some(selected_channel_index) = &self.selected_channel {
										self.guilds[*selected_guild_index].channels[*selected_channel_index].unread = false;
										self.guilds[*selected_guild_index].check_unread();
									}
								}
											
								self.selected_guild = None;
								self.selected_channel = None;
							}
							for i in 0..self.guilds.len() {
								if ui.add(egui::SelectableLabel::new(self.selected_guild == Some(i), self.guilds[i].display())).clicked() {
								
									if let Some(selected_guild_index) = &self.selected_guild {
										if let Some(selected_channel_index) = &self.selected_channel {
											self.guilds[*selected_guild_index].channels[*selected_channel_index].unread = false;
											self.guilds[*selected_guild_index].check_unread();
										}
									}
									
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
										if let Some(selected_channel_index) = &self.selected_channel {
											self.guilds[*selected_guild_index].channels[*selected_channel_index].unread = false;
											self.guilds[*selected_guild_index].check_unread();
										}
										self.selected_channel = None;
									}
									for i in 0..self.guilds[*selected_guild_index].channels.len() {
										if ui.add(egui::SelectableLabel::new(self.selected_channel == Some(i), self.guilds[*selected_guild_index].channels[i].display())).clicked() {
											
											if let Some(selected_channel_index) = &self.selected_channel {
												self.guilds[*selected_guild_index].channels[*selected_channel_index].unread = false;
												self.guilds[*selected_guild_index].check_unread();
											}
											
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
								ui.horizontal( |ui| {
									ui.colored_label(hex_str_to_color("#3399ff"), &message.author_name);
									if let Ok(timestamp) = DateTime::parse_from_rfc2822(&message.timestamp) {
										ui.label(timestamp.format("%H:%M (%a, %e %b)").to_string());
									}
								});
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

pub fn hex_str_to_color(hex_str: &str) -> egui::Color32 {
	egui::Color32::from_hex(hex_str).unwrap_or_else(|_| egui::Color32::WHITE)
}