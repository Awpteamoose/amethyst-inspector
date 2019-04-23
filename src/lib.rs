use amethyst::{
	core::{transform::Transform, Named},
	ecs::prelude::*,
	renderer::{Hidden, HiddenPropagate, Blink}
};
pub use amethyst_imgui::imgui;
pub use paste;
use std::any::Any;

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
			ui.window(imgui::im_str!("Hierarchy")).build(move || {
				fn render_boy<UserData: Default + Any>(
					entity: Entity,
					hierarchy: &amethyst::core::ParentHierarchy,
					names: &ReadStorage<'_, amethyst::core::Named>,
					ui: &imgui::Ui<'_>,
					inspector_state: &mut InspectorState<UserData>,
				) {
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
							if ui.small_button(imgui::im_str!("inspect##selector{:?}", entity)) {
								inspector_state.selected = Some(entity);
							}
							for child in children {
								render_boy(*child, hierarchy, names, ui, inspector_state);
							}
						});

					if !opened {
						ui.same_line(0.);
						if ui.small_button(imgui::im_str!("inspect##selector{:?}", entity)) {
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

pub trait Inspect<'a>: Component {
	type UserData;
	const can_add: bool = false;
	const can_remove: bool = true;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData) {}
	fn add(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {}
	fn setup(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {}
}

impl<'a> Inspect<'a> for Named {
	type UserData = &'a mut dyn Any;
	const can_add: bool = true;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };
		let mut buf = imgui::ImString::new(me.name.clone());
		ui.input_text(imgui::im_str!("Entity {}/{}##named", entity.id(), entity.gen().id()), &mut buf)
			.resize_buffer(true)
			.build();
		me.name = std::borrow::Cow::from(String::from(buf.to_str()));
	}

	fn add(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
		storage.insert(entity, Named::new(format!("Entity {}/{}", entity.id(), entity.gen().id()))).unwrap();
	}
}

impl<'a> Inspect<'a> for Transform {
	type UserData = &'a mut dyn Any;
	const can_add: bool = true;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };

		{
			let translation = me.translation();
			let mut v: [f32; 3] = [translation[0], translation[1], translation[2]];
			ui.drag_float3(imgui::im_str!("translation##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			me.set_translation(v.into());
		}

		{
			let mut rotation = me.rotation().euler_angles().2.to_degrees();
			if rotation == -180. {
				rotation = 180.;
			}
			ui.drag_float(
				imgui::im_str!("rotation##transform{:?}", entity),
				&mut rotation,
			)
			.speed(0.25)
			.build();
			me.set_rotation_2d(rotation.to_radians());
		}

		{
			let scale = me.scale().xy();
			let mut v: [f32; 2] = [scale[0], scale[1]];
			ui.drag_float2(imgui::im_str!("scale##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			me.set_scale(v[0], v[1], 1.);
		}
	}

	fn add(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
		storage.insert(entity, Transform::default());
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

	fn setup(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		user_data.set_max_sprites(me.0 as i32);
	}
}

impl<'a> Inspect<'a> for amethyst::renderer::SpriteRender {
	type UserData = &'a mut dyn MaxSprites;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData) {
		let mut me = if let Some(x) = storage.get_mut(entity) { x } else { return; };

		let mut sprite_number = me.sprite_number as i32;
		ui.slider_int(
			imgui::im_str!("# sprite##sprite_render{:?}", entity),
			&mut sprite_number,
			0,
			user_data.max_sprites(),
		)
		.build();
		me.sprite_number = sprite_number as usize;
	}
}

impl<'a> Inspect<'a> for amethyst::renderer::Rgba {
	type UserData = &'a mut Any;
	const can_add: bool = true;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		use amethyst::renderer::Rgba;

		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };

		let mut v: [f32; 4] = [me.0, me.1, me.2, me.3];
		ui.drag_float4(imgui::im_str!("colour tint##rgba{:?}", entity), &mut v)
			.speed(0.1)
			.build();
		std::mem::replace(me, v.into());
	}

	fn add(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
		storage.insert(entity, amethyst::renderer::Rgba::white()).unwrap();
	}
}

impl<'a> Inspect<'a> for amethyst::renderer::Blink {
	type UserData = &'a mut Any;
	const can_add: bool = true;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		use amethyst::renderer::Rgba;

		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };
		ui.drag_float(
			imgui::im_str!("delay##blink{:?}", entity),
			&mut me.delay,
		).speed(0.1).build();
		ui.checkbox(imgui::im_str!("absolute time##blink{:?}", entity), &mut me.absolute_time);
	}

	fn add(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
		storage.insert(entity, Blink { delay: 0.5, timer: 0., absolute_time: false }).unwrap();
	}
}

#[macro_export]
macro_rules! inspect_marker {
	($cmp: ident) => {
		impl<'a> Inspect<'a> for $cmp {
			type UserData = &'a mut dyn std::any::Any;
			const can_add: bool = true;

			fn add(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
				storage.insert(entity, $cmp).unwrap();
			}
		}
	};
}

inspect_marker!(Hidden);
inspect_marker!(HiddenPropagate);

#[macro_export]
macro_rules! inspector {
	($user_data:ident, $($cmp:ident),+$(,)*) => {
		#[derive(Default)]
		#[allow(missing_copy_implementations)]
		pub struct Inspector;
		impl<'s> System<'s> for Inspector {
			type SystemData = (
				Write<'s, $crate::InspectorState<$user_data>>,
				$(WriteStorage<'s, $cmp>,)+
			);

			$crate::paste::item! {
				#[allow(non_snake_case)]
				fn run(&mut self, (mut inspector_state, $(mut [<hello $cmp>],)+): Self::SystemData) {
					amethyst_imgui::with(move |ui| {
						use $crate::imgui;
						use $crate::Inspect;
						ui.window(imgui::im_str!("Inspector"))
							.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
							.build(move || {
								let entity = if let Some(x) = inspector_state.selected { x } else { return; };
								$($cmp::setup(&mut [<hello $cmp>], entity, &mut inspector_state.user_data);)+

								if ui.collapsing_header(imgui::im_str!("add component")).build() {
									let mut hor_pos = 0.;
									$(
										if $cmp::can_add && ![<hello $cmp>].contains(entity) {
											if ui.small_button(imgui::im_str!("{}", stringify!($cmp))) {
												$cmp::add(&mut [<hello $cmp>], entity, &mut inspector_state.user_data);
											}
											hor_pos += ui.get_item_rect_size().0 + ui.imgui().style().item_spacing.x;
											if hor_pos < ui.get_content_region_avail().0 {
												ui.same_line(0.);
											} else {
												hor_pos = 0.;
											}
										}
									)+
									if hor_pos > 0. {
										ui.new_line();
									}
								}

								$(
									if [<hello $cmp>].contains(entity) {

										let expanded = ui.collapsing_header(imgui::im_str!("{}##header", stringify!($cmp))).flags(imgui::ImGuiTreeNodeFlags::AllowItemOverlap).default_open(true).build();
										if $cmp::can_remove {
											ui.same_line(0.);
											if ui.small_button(imgui::im_str!("remove##{}_header_remove", stringify!($cmp))) {
												[<hello $cmp>].remove(entity);
											}
										}
										if expanded {
											$cmp::inspect(&mut [<hello $cmp>], entity, ui, &mut inspector_state.user_data);
										}
									}
								)+
							});
					});
				}
			}
		}
	};
}
