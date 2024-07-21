use eframe::egui;

pub struct EmojiWindow {
	pub visible: bool,
}

impl EmojiWindow {
	pub fn new() -> Self {
		Self { visible: false }
	}

	pub fn show(&mut self, ctx: &egui::Context) {
		let mut visible = self.visible;
		egui::Window::new("Emojis")
			.open(&mut visible)
			.vscroll(true)
			.hscroll(true)
			.show(ctx, |ui| self.ui(ui));
		self.visible = self.visible && visible;
	}

	fn ui(&mut self, ui: &mut egui::Ui) {
		ui.set_min_width(250.0);
		ui.label("\\(°^°)/");
		ui.label("o(`O´)o");
		ui.label("•`_´•");
		ui.label("( ☉ _ ☉ )");
		ui.label("~(o _ o)~");
		ui.label("~(-■_■)~ ♪♬");
		ui.label("☆ (◕w◕) ☆");
		ui.label("\\(^o^)/");
		ui.label("✌(^o^)✌");
		ui.label("(♥u♥)");
		ui.label("(T_T)");
		ui.label("☭ ♥ ✿ ☢ ☠");
	}
}