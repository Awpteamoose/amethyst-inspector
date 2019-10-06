use crate::prelude::*;

impl<'control, 'resource: 'control> InspectControl<'control, 'resource> for &'control mut std::time::Duration {
	type SystemData = ();
	type Builder = DurationControlBuilder<'control>;
}

pub struct DurationControlBuilder<'control> {
	pub value: &'control mut std::time::Duration,
	pub label: Option<&'control imgui::ImStr>,
	pub speed: f32,
	pub null_to: std::time::Duration,
	pub changed: Option<&'control mut bool>,
}

impl<'control, 'resource: 'control> InspectControlBuilder<'control, 'resource, &'control mut std::time::Duration> for DurationControlBuilder<'control> {
	fn new(value: &'control mut std::time::Duration) -> Self {
		Self { value, label: None, speed: 1., null_to: <std::time::Duration as Default>::default(), changed: None }
	}
	fn label(mut self, label: &'control imgui::ImStr) -> Self {
		self.label = Some(label);
		self
	}
	fn changed(mut self, changed: &'control mut bool) -> Self {
		self.changed = Some(changed);
		self
	}
	fn build(self) {
		amethyst_imgui::with(|ui| {
			let mut v = self.value.as_millis() as i32;
			let mut changed = ui.drag_int(self.label.unwrap(), &mut v).speed(self.speed).min(0).build();
			*self.value = std::time::Duration::from_millis(v as u64);
			if ui.is_item_hovered() && ui.is_mouse_down(imgui::MouseButton::Right) {
				changed = true;
				*self.value = self.null_to;
			}
			if let Some(x) = self.changed { *x = *x || changed };
		});
	}
}

impl<'control> DurationControlBuilder<'control> {
	pub fn speed(mut self, speed: f32) -> Self {
		self.speed = speed;
		self
	}
	pub fn null_to(mut self, null_to: u64) -> Self {
		self.null_to = std::time::Duration::from_millis(null_to);
		self
	}
}
