use amethyst::{
	core::{transform::Transform, Named},
	ecs::prelude::*,
};
use amethyst_imgui::imgui;

#[derive(Default)]
pub struct InspectorState {
	pub selected: Option<Entity>,
}

#[derive(Default, Clone, Copy)]
pub struct InspectorHierarchy;
impl<'s> System<'s> for InspectorHierarchy {
	type SystemData = (
		Write<'s, InspectorState>,
		ReadStorage<'s, amethyst::core::Named>,
		ReadStorage<'s, amethyst::core::Parent>,
		ReadExpect<'s, amethyst::core::ParentHierarchy>,
		Entities<'s>,
	);

	fn run(&mut self, (mut inspector_state, names, parents, hierarchy, entities): Self::SystemData) {
		amethyst_imgui::with(move |ui| {
			ui.window(imgui::im_str!("Hierarchy"))
				.build(move || {
					fn render_boy (entity: Entity, hierarchy: &amethyst::core::ParentHierarchy, names: &ReadStorage<'_, amethyst::core::Named>, ui: &imgui::Ui<'_>, inspector_state: &mut InspectorState) {
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

pub trait Inspect {
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>);
}

impl Inspect for Named {
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>) {
		let mut buf = imgui::ImString::new(self.name.clone());
		ui.input_text(imgui::im_str!("##named{}{}", entity.id(), entity.gen().id()), &mut buf).resize_buffer(true).build();
		self.name = std::borrow::Cow::from(String::from(buf.to_str()));
	}
}

impl Inspect for Transform {
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>) {
		let mut v: [f32; 2] = self.translation().xy().into();
		ui.drag_float2(imgui::im_str!("##transform{}{}", entity.id(), entity.gen().id()), &mut v).build();
		self.set_translation_x(v[0]);
		self.set_translation_y(v[1]);
	}
}

#[macro_export]
macro_rules! inspector {
	($($cmp:ident),+$(,)*) => {
		#[allow(missing_copy_implementations)]
		pub struct Inspector;
		impl<'s> System<'s> for Inspector {
			type SystemData = (
				Read<'s, InspectorState>,
				$(WriteStorage<'s, $cmp>,)+
				Entities<'s>,
			);

			#[allow(non_snake_case)]
			fn run(&mut self, (inspector_state, $(mut $cmp,)+ _entities): Self::SystemData) {
				amethyst_imgui::with(move |ui| {
					ui.window(imgui::im_str!("Inspector"))
						.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
						.build(move || {
							let entity = if let Some(x) = inspector_state.selected { x } else { return; };
							$(
								if let Some(cmp) = $cmp.get_mut(entity) {
									cmp.inspect(entity, ui);
								}
								ui.separator();
							)+
						});
				});
			}
		}
	};
}
