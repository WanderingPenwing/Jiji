use eframe::egui;
use std::{thread, time};


const MAX_FPS : f32 = 30.0;


fn main() -> Result<(), eframe::Error> {

	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([1200.0, 800.0]),
		..Default::default()
	};

	eframe::run_native(
		"Jiji",
		options,
		Box::new(move |_cc| Box::from(Jiji::default())),
	)
}


struct Jiji {
	token: String,
	next_frame: time::Instant,
}


impl Default for Jiji {
	fn default() -> Self {
		Self {
			token: "test".to_string(),
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