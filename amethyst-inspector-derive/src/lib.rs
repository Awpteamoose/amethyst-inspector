extern crate proc_macro;
use proc_macro2::TokenStream;
use proc_quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, DataStruct};
use darling::{FromField, FromDeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inspect))]
struct StructArgs {
	#[darling(default)]
	no_default: bool,
}

#[derive(Debug, FromField, Default)]
#[darling(attributes(inspect), default)]
struct FieldArgs {
	null_to: Option<syn::Lit>,
	speed: Option<f32>,
	skip: bool,
	#[darling(multiple)]
	with_component: Vec<syn::Path>,
}

#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let no_default = StructArgs::from_derive_input(&input).unwrap().no_default;

	let name = input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let (inspect, extra_data) = inspect(&input.data, &name);
	let (can_add, add) = match (no_default, input.data) {
		(_, Data::Struct(DataStruct { fields: Fields::Unit, .. })) => (true, quote!(lazy.insert(entity, Self);)),
		(false, _) => (true, quote!(lazy.insert(entity, Self::default());)),
		(true, _) => (false, quote!({})),
	};

	let expanded = quote! {
		impl<'a> #impl_generics ::amethyst_inspector::Inspect<'a> for #name #ty_generics #where_clause {
			type SystemData = (
				::amethyst::ecs::Read<'a, ::amethyst::ecs::LazyUpdate>,
				::amethyst::ecs::ReadStorage<'a, Self>,
				#extra_data
			);

			#inspect
			fn add((lazy, ..): &mut Self::SystemData, entity: ::amethyst::ecs::Entity) { #add }
			fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { #can_add }
		}
	};

	proc_macro::TokenStream::from(expanded)
}

fn inspect(data: &Data, name: &Ident) -> (TokenStream, TokenStream) {
	match *data {
		Data::Struct(ref data) => {
			match data.fields {
				Fields::Named(ref fields) => {
					let inspect_fields = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip;
						if skip { return quote!(); };

						let name = &f.ident;
						let ty = &f.ty;
						let storage = format!("systemdata_{}", f.ident.as_ref().unwrap());
						let varname = syn::Ident::new(&storage, f.span());

						if !args.with_component.is_empty() {
							return with_component_body(f.ident.as_ref().unwrap(), varname, &args.with_component);
						}

						// TODO: more field attrs
						let null_to = args.null_to.map(|x| quote!(.null_to(#x))).unwrap_or(quote!());
						let speed = args.speed.map(|x| quote!(.speed(#x))).unwrap_or(quote!());

						quote!{
							let mut #name = me.#name.clone();
							<&mut #ty as ::amethyst_inspector::InspectControl>::control(&mut #name)
								.changed(&mut changed)
								.data(#varname)
								#null_to
								#speed
								.label(::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)))
								.build();
						}
					});
					let assign_fields = fields.named.iter().map(|f| {
						if FieldArgs::from_field(&f).unwrap().skip { return quote!(); };
						let name = &f.ident;
						quote!{cmp.#name = #name;}
					});
					let extra_data = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip;
						if skip { return quote!(); };

						if !args.with_component.is_empty() {
							let paths = args.with_component;
							return quote!((
								::amethyst::ecs::Entities<'a>,
								ReadStorage<'a, ::amethyst::core::Named>,
								ReadStorage<'a, ::amethyst::ecs::saveload::U64Marker>,
								#(ReadStorage<'a, #paths>,)*
							),);
						}

						let ty = &f.ty;
						quote!{ <&'a mut #ty as ::amethyst_inspector::InspectControl<'a, 'a>>::SystemData, }
					});
					let extra_data_members = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip;
						if skip { return quote!() };

						let storage = format!("systemdata_{}", f.ident.as_ref().unwrap());
						let varname = syn::Ident::new(&storage, f.span());
						quote!{#varname, }
					});
					(quote! {
						fn inspect((lazy, storage, #(#extra_data_members)*): &mut Self::SystemData, entity: ::amethyst::ecs::Entity) {
							use ::amethyst_inspector::InspectControlBuilder;

							::amethyst_imgui::with(|ui| {
								let me = if let Some(x) = storage.get(entity) { x } else { return; };
								let mut changed = false;
								ui.push_id(::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)));

								#(#inspect_fields)*

								if changed {
									lazy.exec(move |w| {
										if let Some(mut cmp) = w.write_storage::<Self>().get_mut(entity) {
											#(#assign_fields)*
										}
									});
								}
								ui.pop_id();
							});
						}
					}, quote!{#(#extra_data)*})
				}
				Fields::Unit => { (quote!(), quote!()) },
				_ => unimplemented!(),
			}
		},
		_ => unimplemented!(),
	}
}

// TODO: maybe this can be a control-like trait instead of hamfisted any downcasting?
fn with_component_body(name: &syn::Ident, data: syn::Ident, components: &[syn::Path]) -> TokenStream {
	let members: Vec<syn::Index> = components.iter().enumerate().map(|(i, _)| syn::Index::from(i + 3)).collect::<Vec<_>>();
	quote! {
		let mut #name = me.#name.clone();
		{
			use ::amethyst::ecs::{Entity, saveload::{U64Marker as Marker, U64MarkerAllocator as MarkerAllocator}};
			use ::amethyst_imgui::imgui;
			use ::std::any::Any;

			let data = #data;
			let entities = &data.0;
			let named_s = &data.1;
			// TODO: less disgusting, saveload feature
			if let Some(field) = Any::downcast_mut::<Option<Entity>>(&mut #name) {
				let mut current = 0;
				let list = ::std::iter::once(None).chain((entities, #(&data.#members,)*).join().map(|(entity, ..)| Some(entity))).collect::<Vec<_>>();
				let mut items = Vec::<imgui::ImString>::new();
				for (i, &entity) in list.iter().enumerate() {
						if *field == entity { current = i as i32; }

						let label: String = if let Some(entity) = entity {
							if let Some(name) = named_s.get(entity) {
								name.name.to_string()
							} else {
								format!("Entity {}/{}", entity.id(), entity.gen().id())
							}
						} else {
							"None".into()
						};
						items.push(imgui::im_str!("{}", label).into());
				}
				changed = ui.combo(imgui::im_str!("{}", stringify!(#name)), &mut current, items.iter().map(::std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
				*field = list[current as usize];
			} else if let Some(field) = Any::downcast_mut::<Entity>(&mut #name) {
				let mut current = 0;
				let list = (entities, #(&data.#members,)*).join().map(|(entity, ..)| entity).collect::<Vec<_>>();
				let mut items = Vec::<imgui::ImString>::new();
				for (i, &entity) in list.iter().enumerate() {
						if *field == entity { current = i as i32; }

						let label: String = if let Some(name) = named_s.get(entity) {
							name.name.to_string()
						} else {
							format!("Entity {}/{}", entity.id(), entity.gen().id())
						};
						items.push(imgui::im_str!("{}", label).into());
				}
				changed = ui.combo(imgui::im_str!("{}", stringify!(#name)), &mut current, items.iter().map(::std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
				*field = list[current as usize];
			} else if let Some(field) = Any::downcast_mut::<Option<Marker>>(&mut #name) {
				let marker_s = &data.2;

				let mut current = 0;
				let list = ::std::iter::once(None).chain((entities, marker_s, #(&data.#members,)*).join().map(|(entity, marker, ..)| Some((entity, marker)))).collect::<Vec<_>>();
				let mut items = Vec::<imgui::ImString>::new();
				for (i, &item) in list.iter().enumerate() {
						if let Some((entity, marker)) = item {
							if *field == Some(*marker) { current = i as i32; }
						}

						let label: String = if let Some((entity, marker)) = item {
							if let Some(name) = named_s.get(entity) {
								name.name.to_string()
							} else {
								format!("Entity {}/{}", entity.id(), entity.gen().id())
							}
						} else {
							"None".into()
						};
						items.push(imgui::im_str!("{}", label).into());
				}
				changed = ui.combo(imgui::im_str!("{}", stringify!(#name)), &mut current, items.iter().map(::std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
				*field = if let Some((entity, marker)) = list[current as usize] { Some(*marker) } else { None };
			} else if let Some(field) = Any::downcast_mut::<Marker>(&mut #name) {
				let marker_s = &data.2;

				let mut current = 0;
				let list = (entities, marker_s, #(&data.#members,)*).join().map(|(entity, marker, ..)| (entity, marker)).collect::<Vec<_>>();
				let mut items = Vec::<imgui::ImString>::new();
				for (i, &(entity, marker)) in list.iter().enumerate() {
						if *field == *marker { current = i as i32; }

						let label: String = if let Some(name) = named_s.get(entity) {
							name.name.to_string()
						} else {
							format!("Entity {}/{}", entity.id(), entity.gen().id())
						};
						items.push(imgui::im_str!("{}", label).into());
				}
				changed = ui.combo(imgui::im_str!("{}", stringify!(#name)), &mut current, items.iter().map(::std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
				*field = *list[current as usize].1;
			}
		}
	}
}

#[proc_macro_derive(InspectControl, attributes(inspect))]
pub fn derive_inspect_control(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let system_data = match &input.data {
		Data::Struct(data) => {
			match &data.fields {
				Fields::Named(fields) => {
					fields.named.iter().map(|f| {
						if FieldArgs::from_field(&f).unwrap().skip { return quote!(); };

						let ty = &f.ty;
						quote! {<&'control mut #ty as ::amethyst_inspector::InspectControl<'control, 'resource>>::SystemData}
					}).collect::<Vec<_>>()
				},
				_ => unimplemented!(),
			}
		},
		_ => unimplemented!(),
	};

	let control = match input.data {
		Data::Struct(data) => {
			match data.fields {
				Fields::Named(fields) => {
					fields.named.iter().enumerate().map(|(i, f)| {
						let args = FieldArgs::from_field(&f).unwrap();
						if args.skip { return quote!(); };

						let name = &f.ident;

						// TODO: entities/markers
						// if !args.with_component.is_empty() {
						//     return with_component_body(f.ident.as_ref().unwrap(), varname, &args.with_component);
						// }

						// TODO: more field attrs
						let null_to = args.null_to.map(|x| quote!(.null_to(#x))).unwrap_or(quote!());
						let speed = args.speed.map(|x| quote!(.speed(#x))).unwrap_or(quote!());

						let index = syn::Index::from(i);
						quote! {
							<&mut f32 as ::amethyst_inspector::InspectControl>::control(&mut self.value.#name)
								.changed(&mut changed)
								.data(&mut data.#index)
								#null_to
								#speed
								.label(::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)))
								.build();
						}
					}).collect::<Vec<_>>()
				},
				_ => unimplemented!(),
			}
		},
		_ => unimplemented!(),
	};


	let builder = syn::Ident::new(&format!("{}ControlBuilder", name), name.span());
	let expanded = quote! {
		pub struct #builder<'control, 'resource: 'control> {
			pub value: &'control mut #name,
			pub data: Option<&'control mut <&'control mut #name as ::amethyst_inspector::InspectControl<'control, 'resource>>::SystemData>,
			pub label: Option<&'control ::amethyst_imgui::imgui::ImStr>,
			pub changed: Option<&'control mut bool>,
		}

		impl<'control, 'resource: 'control> #impl_generics ::amethyst_inspector::InspectControl<'control, 'resource> for &'control mut #name #ty_generics #where_clause {
			type SystemData = (#(#system_data),*);
			type Builder = #builder<'control, 'resource>;
		}

		impl<'control, 'resource: 'control> ::amethyst_inspector::InspectControlBuilder<'control, 'resource, &'control mut #name> for #builder<'control, 'resource> {
			fn new(value: &'control mut #name) -> Self {
				Self { value, label: None, changed: None, data: None }
			}
			fn label(mut self, label: &'control imgui::ImStr) -> Self {
				self.label = Some(label);
				self
			}
			fn changed(mut self, changed: &'control mut bool) -> Self {
				self.changed = Some(changed);
				self
			}
			fn data(mut self, data: &'control mut <&'control mut #name as ::amethyst_inspector::InspectControl<'control, 'resource>>::SystemData) -> Self {
				self.data = Some(data);
				self
			}
			fn build(mut self) {
				let mut changed = false;
				let mut data = self.data.take().unwrap();

				::amethyst_imgui::with(|ui| {
					ui.tree_node(::amethyst_imgui::imgui::im_str!("{}", stringify!(#name))).selected(true).build(|| {
						#(#control)*
					});
				});

				if let Some(x) = self.changed { *x = *x || changed };
			}
		}
	};

	proc_macro::TokenStream::from(expanded)
}
