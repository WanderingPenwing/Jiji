use eframe::egui;
use image::GenericImageView;
use std::{error::Error, sync::Arc, thread, time};
use tokio::runtime::Runtime;

mod bot;

const MAX_FPS: f32 = 30.0;

fn main() {
	println!("hello there");

	let handle = thread::spawn(|| {
		println!("general kenobi");
        let mut rt = Runtime::new().unwrap();
        rt.block_on(bot::start_discord_bot());
    });

	// Run the GUI on the main thread
	gui();
}

fn gui() {
	let icon_data = load_icon().unwrap_or_default();

	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([1200.0, 800.0])
			.with_icon(Arc::new(icon_data)),
		..Default::default()
	};

	let _ = eframe::run_native("Jiji", options, Box::new(move |_cc| Box::from(Jiji::new())));
}

struct Jiji {
	next_frame: time::Instant,
}

impl Jiji {
	fn new() -> Self {
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

	fn on_exit(&mut self, _gl: std::option::Option<&eframe::glow::Context>) {
		//self.runtime.shutdown_background();
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
