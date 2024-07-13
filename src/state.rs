use eframe::egui;
use image::GenericImageView;
use std::{
	error::Error,
	fs,
	fs::{read_to_string, OpenOptions},
	io::Write,
	path::Path,
};
use serde::Serialize;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AppState {
	pub bot_token: String,
	pub channels_to_notify: Vec<String>,
	pub dm_channels: HashMap<String, String>,
}

impl Default for AppState {
	fn default() -> Self {
		Self {
			bot_token: "".to_string(),
			channels_to_notify: vec![],
			dm_channels: HashMap::new()
		}
	}
}

pub fn save_state(state: &AppState, file_path: &Path) -> Result<(), std::io::Error> {
	let serialized_state = serde_json::to_string(state)?;

	if let Some(parent_dir) = file_path.parent() {
		fs::create_dir_all(parent_dir)?;
	}

	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(file_path)?;

	file.write_all(serialized_state.as_bytes())?;

	println!("state : Saved state at {}", file_path.display());

	Ok(())
}

pub fn load_state(file_path: &Path) -> AppState {
	if let Ok(serialized_state) = read_to_string(file_path) {
		if let Ok(app_state) = serde_json::from_str(&serialized_state) {
			return app_state
		}
	}
	println!("state : failed to load state");
	AppState::default()
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
