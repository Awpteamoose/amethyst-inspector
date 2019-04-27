use amethyst::ecs::prelude::*;
use amethyst_imgui::imgui;

#[derive(Default, Clone, Copy)]
pub struct InspectorHierarchy {
	dragging: Option<Entity>,
	hovering: Option<Entity>,
}

impl InspectorHierarchy {
	fn render_boy(
		&mut self,
		entity: Entity,
		hierarchy: &amethyst::core::ParentHierarchy,
		names: &ReadStorage<'_, amethyst::core::Named>,
		ui: &imgui::Ui<'_>,
		inspector_state: &mut crate::InspectorState,
		entities: &amethyst::ecs::world::EntitiesRes,
		lazy: &LazyUpdate,
	) {
		let children = hierarchy.children(entity);

		let label: String = if let Some(name) = names.get(entity) {
			name.name.to_string()
		} else {
			format!("Entity {}/{}", entity.id(), entity.gen().id())
		};

		macro_rules! tree_node_buttons {
			() => {
				if ui.is_item_hovered_with_flags(imgui::ImGuiHoveredFlags::AllowWhenBlockedByActiveItem) {
					self.hovering = Some(entity);

					if ui.imgui().is_mouse_down(imgui::ImMouseButton::Left) && self.dragging.is_none() {
						self.dragging = Some(entity);
					}
				}
				ui.same_line(0.);
				if ui.small_button(imgui::im_str!("inspect##selector{:?}", entity)) {
					inspector_state.selected = Some(entity);
				}
			};
		}

		let mut opened = false;
		ui.tree_node(imgui::im_str!("{}##{:?}", label, entity))
			.label(imgui::im_str!("{}", label))
			.selected(matches::matches!(inspector_state.selected, Some(x) if x == entity))
			.leaf(children.is_empty())
			.build(|| {
				opened = true;
				tree_node_buttons!();
				for child in children {
					self.render_boy(*child, hierarchy, names, ui, inspector_state, &entities, &lazy);
				}
			});

		if !opened {
			tree_node_buttons!();
		}
	}
}

impl<'s> System<'s> for InspectorHierarchy {
	type SystemData = (
		Write<'s, crate::InspectorState>,
		ReadStorage<'s, amethyst::core::Named>,
		ReadStorage<'s, amethyst::core::Parent>,
		ReadExpect<'s, amethyst::core::ParentHierarchy>,
		Entities<'s>,
		Read<'s, LazyUpdate>,
	);

	fn run(&mut self, (mut inspector_state, names, parents, hierarchy, entities, lazy): Self::SystemData) {
		amethyst_imgui::with(move |ui| {
			ui.window(imgui::im_str!("Hierarchy"))
				.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
				.build(move || {
					self.hovering = None;

					if ui.small_button(imgui::im_str!("new entity##hierarchy")) {
						lazy.create_entity(&entities).build();
					}
					ui.separator();
					for (entity, _) in (&entities, !&parents).join() {
						self.render_boy(entity, &hierarchy, &names, &ui, &mut inspector_state, &entities, &lazy);
					}

					let is_dragging = ui.imgui().is_mouse_dragging(imgui::ImMouseButton::Left);
					let is_mouse_down = ui.imgui().is_mouse_down(imgui::ImMouseButton::Left);
					if let Some(dragged) = self.dragging {
						if !is_dragging && !is_mouse_down {
							if let Some(hover) = self.hovering {
								if dragged != hover {
									lazy.insert(dragged, amethyst::core::Parent::new(hover));
								}
							} else {
								lazy.remove::<amethyst::core::Parent>(dragged);
							}
							self.dragging = None;
						}
					}
				});
		});
	}
}
