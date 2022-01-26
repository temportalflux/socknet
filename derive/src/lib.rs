extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream, Result},
	parse_macro_input, ItemStruct,
};

#[derive(Debug)]
struct BuilderParams {
	handler_id: syn::LitStr,
	stream_type: syn::Variant,
	initiator: Option<(syn::Ident, syn::Ident)>,
	responder: (syn::Ident, syn::Ident),
}
impl Parse for BuilderParams {
	fn parse(input: ParseStream) -> Result<Self> {
		let handler_id = input.parse()?;
		input.parse::<syn::Token![,]>()?;
		let stream_type: syn::Variant = input.parse()?;
		input.parse::<syn::Token![,]>()?;
		let initiator = match stream_type.ident.to_string().as_str() {
			"Datagram" => None,
			_ => {
				let buildable = input.parse()?;
				input.parse::<syn::Token![::]>()?;
				let callback = input.parse()?;
				input.parse::<syn::Token![,]>()?;
				Some((buildable, callback))
			}
		};
		let responder = {
			let buildable = input.parse()?;
			input.parse::<syn::Token![::]>()?;
			let callback = input.parse()?;
			(buildable, callback)
		};
		Ok(Self {
			handler_id,
			stream_type,
			initiator,
			responder,
		})
	}
}

/// #[builder("<handler_name>", <stream::Kind>, <Initiator>::<callback>, <Responder>::<callback>)]
/// #[builder("handshake", Bidirectional, Handshake::process_client, Handshake::process_server)]
/// #[builder("move_player", Datagram, MovePlayer::receive)]
#[proc_macro_attribute]
pub fn builder(args: TokenStream, input: TokenStream) -> TokenStream {
	// tells rust that this macro must annotate a `struct`
	let item_struct = parse_macro_input!(input as ItemStruct);
	let name = &item_struct.ident;
	let BuilderParams {
		handler_id,
		stream_type,
		initiator,
		responder,
	} = parse_macro_input!(args as BuilderParams);
	let (responder_buildable, responder_callback) = responder;

	let (open_function, responder_case) = match stream_type.ident.to_string().as_str() {
		"Unidirectional" => (
			make_open_function(
				syn::Ident::new("open_uni", Span::call_site()),
				initiator.unwrap(),
			),
			quote! { stream::Typed::Unidirectional(recv) => recv },
		),
		"Bidirectional" => (
			make_open_function(
				syn::Ident::new("open_bi", Span::call_site()),
				initiator.unwrap(),
			),
			quote! { stream::Typed::Bidirectional(send, recv) => (send, recv) },
		),
		"Datagram" => (
			quote! {
				fn open(self: Arc<Self>, connection: Weak<Connection>) -> Result<()> {
					unimplemented!()
				}
			}
			.into(),
			quote! { stream::Typed::Datagram(bytes) => bytes },
		),
		_ => unimplemented!(),
	};
	let open_function: proc_macro2::TokenStream = open_function.into();

	let gen = quote! {
		#item_struct

		impl stream::Builder for #name {
			fn unique_id() -> &'static str {
				#handler_id
			}

			#open_function

			fn build(
				self: Arc<Self>,
				connection: Arc<Connection>,
				stream: stream::Typed,
			) -> Result<()> {
				let kind = stream::Handler::Responder;
				let log_target = format!(
					"{}/{}[{}]",
					kind,
					Self::unique_id(),
					connection.remote_address()
				);
				stream::spawn(log_target, async move {
					use stream::Buildable;
					let stream_impl = #responder_buildable::build(self, connection, match stream {
						#responder_case,
						_ => unimplemented!(),
					});
					stream_impl.#responder_callback().await?;
					Ok(())
				});
				Ok(())
			}

		}
	};
	gen.into()
}

fn make_open_function(
	connection_open: syn::Ident,
	(buildable, callback): (syn::Ident, syn::Ident),
) -> TokenStream {
	quote! {
		fn open(self: Arc<Self>, connection: Weak<Connection>) -> Result<()> {
			let kind = stream::Handler::Initiator;
			let connection = Connection::upgrade(&connection)?;
			let log_target = format!(
				"{}/{}[{}]",
				kind,
				Self::unique_id(),
				connection.remote_address()
			);
			stream::spawn(log_target, async move {
				use stream::Buildable;
				let stream = connection.#connection_open().await?;
				let stream_impl = #buildable::build(self, connection, stream);
				stream_impl.#callback().await?;
				Ok(())
			});
			Ok(())
		}
	}
	.into()
}

#[derive(Debug)]
struct PacketKindArgs {
	socknet_crate_path: syn::ExprPath,
}

impl Parse for PacketKindArgs {
	fn parse(input: ParseStream) -> Result<Self> {
		let socknet_crate_path = input.parse()?;
		Ok(Self { socknet_crate_path })
	}
}

#[proc_macro_attribute]
pub fn packet_kind(args: TokenStream, input: TokenStream) -> TokenStream {
	// tells rust that this macro must annotate a `struct`
	let item_struct = parse_macro_input!(input as ItemStruct);
	let name = &item_struct.ident;

	// ensure the `#[packet_kind]` macro has 2 specific arguments
	let PacketKindArgs { socknet_crate_path } = parse_macro_input!(args as PacketKindArgs);
	let unique_id = format!("{}", name);

	// Construct the final metaprogramming,
	// implementing the `packet::Kind` and `Registerable<KindId, Registration>` traits for the struct.
	return quote! {
		#item_struct

		impl #socknet_crate_path::packet::Registerable<
			#socknet_crate_path::packet::KindId,
			#socknet_crate_path::packet::Registration
		> for #name {
			fn unique_id() -> #socknet_crate_path::packet::KindId {
				#unique_id
			}
			fn registration() -> #socknet_crate_path::packet::Registration
			where
				Self: Sized + 'static,
			{
				#socknet_crate_path::packet::Registration::of::<Self>()
			}
		}
		impl #socknet_crate_path::packet::Kind for #name {
			fn serialize_to(&self) -> Vec<u8> {
				use #socknet_crate_path::packet::Registerable;
				profiling::scope!("packet-serialize", #name::unique_id());
				#socknet_crate_path::serde::to_vec(&self).unwrap()
			}
			fn deserialize_from(bytes: &[u8]) -> Box<dyn std::any::Any + 'static + Send>
			where
				Self: Sized,
			{
				use #socknet_crate_path::packet::Registerable;
				profiling::scope!("packet-deserialize", #name::unique_id());
				Box::new(#socknet_crate_path::serde::from_read_ref::<[u8], #name>(&bytes).unwrap())
			}
		}
	}
	.into();
}
