#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index, Ident, DataStruct, Type};

#[proc_macro_derive(Inspect)]
pub fn derive_inspect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let inspect = inspect(&input.data, &name);
	let add = match input.data {
		Data::Struct(DataStruct { fields: Fields::Unit, .. }) => quote!(Self),
		_ => quote!(Self::default()),
	};

	let expanded = quote! {
		impl<'a> #impl_generics ::amethyst_inspector::Inspect<'a> for #name #ty_generics #where_clause {
			type SystemData = (
				::amethyst::ecs::ReadStorage<'a, Self>,
				::amethyst::ecs::Read<'a, ::amethyst::ecs::LazyUpdate>,
			);

			const CAN_ADD: bool = true;

			fn inspect((storage, lazy): &Self::SystemData, entity: ::amethyst::ecs::Entity, ui: &::amethyst_imgui::imgui::Ui<'_>) { #inspect }
			fn add((storage, lazy): &Self::SystemData, entity: ::amethyst::ecs::Entity) { lazy.insert(entity, #add); }
		}
	};

	proc_macro::TokenStream::from(expanded)
}

fn inspect(data: &Data, name: &Ident) -> TokenStream {
	match *data {
		Data::Struct(ref data) => {
			match data.fields {
				Fields::Named(ref fields) => {
					let recurse = fields.named.iter().map(|f| {
						let name = &f.ident;
						let ty = &f.ty;
						quote_spanned! {f.span()=>
							changed = <#ty as ::amethyst_inspector::InspectControl>::control(&mut new_me.#name, ::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)), ui) || changed;
						}
					});
					quote! {
						let me = if let Some(x) = storage.get(entity) { x } else { return; };
						let mut new_me = me.clone();
						let mut changed = false;
						ui.push_id(::amethyst_imgui::imgui::im_str!("#name"));

						#(#recurse)*

						if changed {
							lazy.insert(entity, new_me);
						}
						ui.pop_id();
					}
				}
				Fields::Unit => { quote!() },
				_ => unimplemented!(),
			}
		},
		_ => unimplemented!(),
	}
}
