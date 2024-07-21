use eframe::egui;
use chrono::{DateTime, Local};

use crate::postman;
use crate::Jiji;

const MESSAGE_EDIT_ROWS : usize = 4;

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
							ui.vertical(|ui| {
								if ui.button(">").clicked() && self.current_message != "" {
									let _ = self.sender.send(postman::Packet::SendMessage(self.guilds[guild_index].channels[channel_index].id.clone(), self.current_message.clone()));
									self.current_message = "".to_string();
								}
								if ui.button("#").clicked() {
									self.emoji_window.visible = !self.emoji_window.visible;
								}
							});
							egui::ScrollArea::vertical()
								.show(ui, |ui| {
									let _response = ui.add(egui::TextEdit::multiline(&mut self.current_message)
										.desired_width(f32::INFINITY)
										.desired_rows(MESSAGE_EDIT_ROWS)
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
			if let Some(selected_guild_index) = &self.selected_guild {
				if let Some(selected_channel_index) = &self.selected_channel {
					let selected_guild = &mut self.guilds[*selected_guild_index];
					let selected_channel = &mut selected_guild.channels[*selected_channel_index];
					
					let scrollarea_result = egui::ScrollArea::vertical()
						.stick_to_bottom(true)
						.vertical_scroll_offset(selected_channel.scroll_offset)
						.show(ui, |ui| {
						
						let mut last_author = "".to_string();
						
						if selected_channel.messages.len() < 2 {
							return
						}
						
						for message in &selected_channel.messages {							
							if message.author_name == "+" {
								if ui.button("+").clicked() {
									let _ = self.sender.send(postman::Packet::FetchMessages(
										selected_guild.id.clone(), 
										selected_channel.id.clone(), 
										selected_channel.messages[1].id.clone(),
									));
									self.pending_bot_requests += 1;
								}
								continue
							}
							if message.author_name != last_author {
								ui.separator();
								ui.horizontal( |ui| {
									ui.colored_label(hex_str_to_color("#3399ff"), &message.author_name);
									if let Ok(timestamp) = DateTime::parse_from_rfc2822(&message.timestamp) {
										let local_timestamp = timestamp.with_timezone(&Local);
										ui.label(local_timestamp.format("%H:%M (%a, %e %b)").to_string());
									}
								});
							} else {
								ui.label("");
							}
							ui.label(&message.content);
							last_author = message.author_name.clone();
						}
					});
					
					let new_content_size = scrollarea_result.content_size[1];
					let new_inner_size = scrollarea_result.inner_rect.max.y - scrollarea_result.inner_rect.min.y;
					
					if selected_channel.registered_messages != selected_channel.messages.len() {
						if selected_channel.scroll_offset >= selected_channel.content_size - selected_channel.inner_size * 1.5 {
							selected_channel.scroll_offset = new_content_size - new_inner_size; 
						} else if selected_channel.scroll_offset < 1.0 {
							selected_channel.scroll_offset = new_content_size - selected_channel.content_size;
						}
						
						selected_channel.registered_messages = selected_channel.messages.len();
						self.redraw = true;
						
					} else {
						selected_channel.scroll_offset = scrollarea_result.state.offset[1];
					}
					
					selected_channel.content_size = new_content_size;
					selected_channel.inner_size = new_inner_size;
				}
			}
		});
	}
}

pub fn hex_str_to_color(hex_str: &str) -> egui::Color32 {
	egui::Color32::from_hex(hex_str).unwrap_or_else(|_| egui::Color32::WHITE)
}