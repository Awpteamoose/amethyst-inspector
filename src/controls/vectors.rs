macro_rules! vectors {
	($([$size:tt $type:ident $kind:ident]),+$(,)*) => {$(paste::item!{
		mod [<$kind$type$size>] {
			use crate::prelude::*;

			impl<'small, 'big: 'small> InspectControl<'small, 'big> for &'small mut [<Vector$size>]<$type> {
				type SystemData = ();
				type Builder = Builder<'small>;
			}

			pub struct Builder<'small> {
				pub value: &'small mut [<Vector$size>]<$type>,
				pub label: Option<&'small imgui::ImStr>,
				pub speed: f32,
				pub null_to: $type,
				pub changed: Option<&'small mut bool>,
			}

			impl<'small, 'big: 'small> InspectControlBuilder<'small, 'big, &'small mut [<Vector$size>]<$type>> for Builder<'small> {
				fn new(value: &'small mut [<Vector$size>]<$type>) -> Self {
					Self { value, label: None, speed: 1., null_to: <$type as Default>::default(), changed: None }
				}
				fn label(mut self, label: &'small imgui::ImStr) -> Self {
					self.label = Some(label);
					self
				}
				fn changed(mut self, changed: &'small mut bool) -> Self {
					self.changed = Some(changed);
					self
				}
				fn build(self) {
					amethyst_imgui::with(|ui| {
						let mut changed = false;
						let label = self.label.unwrap();
						ui.push_id(label);

						let spacing = ui.imgui().style().item_inner_spacing.x;
						let width = ((ui.get_window_size().0 - spacing * (($size - 1) as f32 * 1.5)) * 0.65) / $size as f32;

						for i in 0 .. $size {
							ui.with_id(i as i32, || {
								ui.with_item_width(width, || {
									let mut v = self.value[i as usize] as _;
									changed = ui.[<drag_$kind>](im_str!(""), &mut v).speed(self.speed).min(std::$type::MIN as _).max(std::$type::MAX as _).build() || changed;
									self.value[i as usize] = v as _;
									if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
										changed = true;
										self.value[i as usize] = self.null_to;
									}
									ui.same_line_spacing(0., spacing);
								});
							});
						}

						ui.text(label);
						ui.pop_id();
						if let Some(x) = self.changed { *x = *x || changed };
					});
				}
			}

			impl<'small> Builder<'small> {
				pub fn speed(mut self, speed: f32) -> Self {
					self.speed = speed;
					self
				}
				pub fn null_to(mut self, null_to: $type) -> Self {
					self.null_to = null_to;
					self
				}
			}
		}
	})+};
}

vectors![
	[2 f32 float],
	[3 f32 float],
	[4 f32 float],
	[2 f64 float],
	[3 f64 float],
	[4 f64 float],
	[2 u8 int],
	[3 u8 int],
	[4 u8 int],
	[2 u16 int],
	[3 u16 int],
	[4 u16 int],
	[2 u32 int],
	[3 u32 int],
	[4 u32 int],
	[2 u64 int],
	[3 u64 int],
	[4 u64 int],
	[2 usize int],
	[3 usize int],
	[4 usize int],
	[2 i8 int],
	[3 i8 int],
	[4 i8 int],
	[2 i16 int],
	[3 i16 int],
	[4 i16 int],
	[2 i32 int],
	[3 i32 int],
	[4 i32 int],
	[2 i64 int],
	[3 i64 int],
	[4 i64 int],
];
