## About:
Unity-inspired entity hierarchy and component editor via [amethyst-imgui](https://github.com/Awpteamoose/amethyst-imgui)

## Usage:
1. Add `InspectorHierarchy` and `Inspector` to your systems
```rust
	.with(InspectorHierarchy, "inspector_hierarchy", &[])
	.with(Inspector, "inspector", &["inspector_hierarchy"])
```
2. Implement `Inspect` for all components that you want to show up in the inspector. For example:
```rust
impl Inspect for Transform {
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>) {
		let mut v: [f32; 2] = self.translation().xy().into();
		ui.drag_float2(imgui::im_str!("##transform{}{}", entity.id(), entity.gen().id()), &mut v).build();
		self.set_translation_x(v[0]);
		self.set_translation_y(v[1]);
	}
}
```
3. List all components you want to show up in the inspector in the `inspector!` macro. This creates a system called `Inspector`.
```rust
inspector![
	Named,
	Transform,
];
```
