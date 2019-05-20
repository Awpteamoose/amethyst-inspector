[![](https://img.shields.io/badge/docs-gh--pages-yellowgreen.svg)](https://awpteamoose.github.io/amethyst-inspector/amethyst_inspector/index.html)

## About:
Unity-inspired entity hierarchy and component editor via [amethyst-imgui](https://github.com/Awpteamoose/amethyst-imgui)

## Basic sage:
1. `#[derive(Inspect)]` on all components that you want to show up in the inspector. For example:
```rust
// InspectControl is a derive for drawing nested structs
#[derive(Clone, InspectControl)]
pub struct Movement {
	// null_to is what the field is set to on right click
	// speed is how fast the slider can be dragged
	#[inspect(null_to = 10., speed = 0.1)]
	pub speed: f32,
	pub direction: Vector2<f32>,
}

#[derive(Component, Clone, Inspect)]
// #[inspect(no_default)] would disable adding this component
pub struct Player {
	// will only show a dropdown for entities with this component
	// also works for non-option Entity (however that can't be defaulted), U64Marker, Option<U64Marker>
	#[inspect(with_component = "cmp::Location")]
	pub location: Option<Entity>,
	pub movement: Movement,
	// similar to serde(skip) - don't create a control for this field
	#[inspect(skip)]
	pub schlonk: Schlonker,
}
```
2. List all your components you want to show up in the inspector with an `inspector!` macro. This creates a system called `Inspector`.
```rust
inspector![
	Named,
	Transform,
	Rgba,
];
```
3. Add `InspectorHierarchy` and `Inspector` systems.
```rust
	.with(amethyst_inspector::InspectorHierarchy::<UserData>::default(), "", &[])
	.with(Inspector, "", &[""])
```

# Help wanted
Drop me a line on discord or create an issue if you can help or have advice:

* Make a modal or smth specifying properties when adding components
* documentation
* tests or smth idk if applicable

![screenshot](https://raw.githubusercontent.com/awpteamoose/amethyst-inspector/master/screenshot.png)
