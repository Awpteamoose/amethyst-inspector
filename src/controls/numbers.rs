macro_rules! numbers {
	($([$type:ident $kind:ident]),+$(,)*) => {$(paste::item!{
		mod $type {
			use crate::prelude::*;

			impl<'a> InspectControl<'a> for $type {
				type SystemData = ();
				type Builder = Builder<'a>;
			}

			pub struct Builder<'a> {
				pub value: &'a mut $type,
				pub label: Option<&'a imgui::ImStr>,
				pub speed: f32,
				pub null_to: $type,
				pub changed: Option<&'a mut bool>,
			}

			impl<'a> InspectControlBuilder<'a, $type> for Builder<'a> {
				fn new(value: &'a mut $type) -> Self {
					Self { value, label: None, speed: 1., null_to: <$type as Default>::default(), changed: None }
				}
			}

			impl<'a> Builder<'a> {
				pub fn label(mut self, label: &'a imgui::ImStr) -> Self {
					self.label = Some(label);
					self
				}
				pub fn speed(mut self, speed: f32) -> Self {
					self.speed = speed;
					self
				}
				pub fn null_to(mut self, null_to: $type) -> Self {
					self.null_to = null_to;
					self
				}
				pub fn changed(mut self, changed: &'a mut bool) -> Self {
					self.changed = Some(changed);
					self
				}
				pub fn build(self) {
					amethyst_imgui::with(|ui| {
						let mut v = *self.value as _;
						let mut changed = ui.[<drag_$kind>](self.label.unwrap(), &mut v).speed(self.speed).min(std::$type::MIN as _).max(std::$type::MAX as _).build();
						*self.value = v as _;
						if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
							changed = true;
							*self.value = self.null_to;
						}
						if let Some(x) = self.changed { *x = *x || changed };
					});
				}
			}
		}
	})+};
}

numbers![
	[u8 int],
	[u16 int],
	[u32 int],
	[u64 int],
	[usize int],
	[i8 int],
	[i16 int],
	[i32 int],
	[i64 int],
	[f32 float],
	[f64 float],
];
