use amethyst::{
	core::{transform::Transform, Named},
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use std::any::Any;
pub use paste;

#[derive(Default)]
pub struct InspectorState<UserData: Default + Any> {
	pub selected: Option<Entity>,
	pub user_data: UserData,
}

#[derive(Default, Clone, Copy)]
pub struct InspectorHierarchy<UserData> {
	_pd: std::marker::PhantomData<UserData>,
}
impl<'s, UserData: 'static + Sync + Send + Default + Any> System<'s> for InspectorHierarchy<UserData> {
	type SystemData = (
		Write<'s, InspectorState<UserData>>,
		ReadStorage<'s, amethyst::core::Named>,
		ReadStorage<'s, amethyst::core::Parent>,
		ReadExpect<'s, amethyst::core::ParentHierarchy>,
		Entities<'s>,
	);

	fn run(&mut self, (mut inspector_state, names, parents, hierarchy, entities): Self::SystemData) {
		amethyst_imgui::with(move |ui| {
			ui.window(imgui::im_str!("Hierarchy"))
				.build(move || {
					fn render_boy<UserData: Default + Any>(entity: Entity, hierarchy: &amethyst::core::ParentHierarchy, names: &ReadStorage<'_, amethyst::core::Named>, ui: &imgui::Ui<'_>, inspector_state: &mut InspectorState<UserData>) {
						let children = hierarchy.children(entity);

						let label: imgui::ImString = if let Some(name) = names.get(entity) {
							imgui::im_str!("{}", name.name).into()
						} else {
							imgui::im_str!("Entity {}/{}", entity.id(), entity.gen().id()).into()
						};

						let mut opened = false;
						ui.tree_node(&label)
							.selected(matches::matches!(inspector_state.selected, Some(x) if x == entity))
							.leaf(children.is_empty())
							.build(|| {
								opened = true;
								ui.same_line(0.);
								if ui.small_button(imgui::im_str!("inspect##{:?}_selector", &label)) {
									inspector_state.selected = Some(entity);
								}
								for child in children {
									render_boy(*child, hierarchy, names, ui, inspector_state);
								}
							});

						if !opened {
							ui.same_line(0.);
							if ui.small_button(imgui::im_str!("inspect##{:?}_selector", &label)) {
								inspector_state.selected = Some(entity);
							}
						}
					};

					for (entity, _) in (&entities, !&parents).join() {
						render_boy(entity, &hierarchy, &names, &ui, &mut inspector_state);
					}
				});
		});
	}
}

pub trait Inspect<'a> {
	type UserData;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData);
}

impl<'a> Inspect<'a> for Named {
	type UserData = &'a mut dyn Any;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		let mut buf = imgui::ImString::new(self.name.clone());
		ui.input_text(imgui::im_str!("name##named{}{}", entity.id(), entity.gen().id()), &mut buf).resize_buffer(true).build();
		self.name = std::borrow::Cow::from(String::from(buf.to_str()));
		ui.separator();
	}
}

impl<'a> Inspect<'a> for Transform {
	type UserData = &'a mut dyn Any;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		{
			let translation = self.translation();
			let mut v: [f32; 3] = [translation[0], translation[1], translation[2]];
			ui.drag_float3(imgui::im_str!("translation##transform{}{}", entity.id(), entity.gen().id()), &mut v).speed(0.1).build();
			self.set_translation(v.into());
		}

		{
			let mut rotation = self.rotation().euler_angles().2.to_degrees();
			if rotation == -180. { rotation = 180.; }
			ui.drag_float(imgui::im_str!("rotation##transform{}{}", entity.id(), entity.gen().id()), &mut rotation).speed(0.25).build();
			self.set_rotation_2d(rotation.to_radians());
		}

		{
			let scale = self.scale().xy();
			let mut v: [f32; 2] = [scale[0], scale[1]];
			ui.drag_float2(imgui::im_str!("scale##transform{}{}", entity.id(), entity.gen().id()), &mut v).speed(0.1).build();
			self.set_scale(v[0], v[1], 1.);
		}
		ui.separator();
	}
}

// TODO: rework these so it can be a non-contiguous array of sprites etc
pub trait MaxSprites {
	fn max_sprites(&self) -> i32;
	fn set_max_sprites(&mut self, value: i32);
}

pub struct SpriteInfo(pub u32);

impl Component for SpriteInfo {
	type Storage = DenseVecStorage<Self>;
}

impl<'a> Inspect<'a> for SpriteInfo {
	type UserData = &'a mut dyn MaxSprites;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData) {
		user_data.set_max_sprites(self.0 as i32);
	}
}

impl<'a> Inspect<'a> for amethyst::renderer::SpriteRender {
	type UserData = &'a mut dyn MaxSprites;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData) {
		let mut sprite_number = self.sprite_number as i32;
		ui.slider_int(imgui::im_str!("# sprite##sprite_render{}{}", entity.id(), entity.gen().id()), &mut sprite_number, 0, user_data.max_sprites()).build();
		self.sprite_number = sprite_number as usize;
		ui.separator();
	}
}

impl<'a> Inspect<'a> for amethyst::renderer::Rgba {
	type UserData = &'a mut Any;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData) {
		use amethyst::renderer::Rgba;

		let mut v: [f32; 4] = [self.0, self.1, self.2, self.3];
		ui.drag_float4(imgui::im_str!("colour tint##rgba{}{}", entity.id(), entity.gen().id()), &mut v).speed(0.1).build();
		std::mem::replace(self, v.into());
		ui.separator();
	}
}

#[macro_export]
macro_rules! inspector {
	($user_data:ident, $($cmp:ident),+$(,)*) => {
		#[derive(Default)]
		#[allow(missing_copy_implementations)]
		pub struct Inspector;
		impl<'s> System<'s> for Inspector {
			type SystemData = (
				Write<'s, InspectorState<$user_data>>,
				$(WriteStorage<'s, $cmp>,)+
			);

			$crate::paste::item! {
				#[allow(non_snake_case)]
				fn run(&mut self, (mut inspector_state, $(mut [<hello $cmp>],)+): Self::SystemData) {
					amethyst_imgui::with(move |ui| {
						ui.window(imgui::im_str!("Inspector"))
							.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
							.build(move || {
								let entity = if let Some(x) = inspector_state.selected { x } else { return; };
								$(
									if let Some(cmp) = [<hello $cmp>].get_mut(entity) {
										cmp.inspect(entity, ui, &mut inspector_state.user_data);
									}
								)+
							});
					});
				}
			}
		}
	};
}
