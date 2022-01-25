extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream, Result},
	parse_macro_input, ItemStruct,
};

#[derive(Debug)]
struct InitiatorParams(syn::Variant, syn::Ident);
impl Parse for InitiatorParams {
	fn parse(input: ParseStream) -> Result<Self> {
		let stream_kind = input.parse()?;
		input.parse::<syn::Token![,]>()?;
		let callback = input.parse()?;
		Ok(Self(stream_kind, callback))
	}
}

#[proc_macro_attribute]
pub fn initiator(args: TokenStream, input: TokenStream) -> TokenStream {
	// tells rust that this macro must annotate a `struct`
	let item_struct = parse_macro_input!(input as ItemStruct);
	let name = &item_struct.ident;
	let params = parse_macro_input!(args as InitiatorParams);

	let callback = params.1;
	let (stream_type, connection_open) = match params.0.ident.to_string().as_str() {
		"Unidirectional" => (
			quote! { stream::Send },
			syn::Ident::new("open_uni", Span::call_site()),
		),
		"Bidirectional" => (
			quote! { (stream::Send, stream::Recv) },
			syn::Ident::new("open_bi", Span::call_site()),
		),
		"Datagram" => unimplemented!(),
		_ => unimplemented!(),
	};

	let gen = quote! {
		#item_struct

		impl stream::Initiator<#stream_type> for #name {
			fn open(connection: &std::sync::Arc<Connection>) -> stream::Result<()> {
				let async_conn = std::sync::Arc::downgrade(&connection);
				connection.spawn(async move {
					let connection = Connection::upgrade(&async_conn)?;
					let stream = connection.#connection_open().await?;
					let stream_impl = Self::new(async_conn, stream);
					stream_impl.#callback().await?;
					Ok(())
				});
				Ok(())
			}
		}
	};
	gen.into()
}

#[derive(Debug)]
struct ResponderParams(syn::LitStr, InitiatorParams);
impl Parse for ResponderParams {
	fn parse(input: ParseStream) -> Result<Self> {
		let stream_kind = input.parse()?;
		input.parse::<syn::Token![,]>()?;
		let initiator = input.parse()?;
		Ok(Self(stream_kind, initiator))
	}
}

#[proc_macro_attribute]
pub fn responder(args: TokenStream, input: TokenStream) -> TokenStream {
	// tells rust that this macro must annotate a `struct`
	let item_struct = parse_macro_input!(input as ItemStruct);
	let name = &item_struct.ident;
	let params = parse_macro_input!(args as ResponderParams);

	let registerable_id = params.0;
	let callback = params.1 .1;
	let (stream_type, receiver_case) = match params.1 .0.ident.to_string().as_str() {
		"Unidirectional" => (
			quote! { stream::Recv },
			quote! { stream::Typed::Unidirectional(recv) => #name::receive(connection, recv) },
		),
		"Bidirectional" => (
			quote! { (stream::Send, stream::Recv) },
			quote! { stream::Typed::Bidirectional(send, recv) => #name::receive(connection, (send, recv)) },
		),
		"Datagram" => (
			quote! { stream::Bytes },
			quote! { stream::Typed::Datagram(bytes) => #name::receive(connection, bytes) },
		),
		_ => unimplemented!(),
	};

	let gen = quote! {
		#item_struct

		impl stream::processor::Registerable for #name {
			fn unique_id() -> &'static str {
				#registerable_id
			}

			fn create_receiver(connection: std::sync::Weak<Connection>, stream: stream::Typed) -> stream::Result<()> {
				use stream::Responder;
				match stream {
					#receiver_case,
					_ => unimplemented!(),
				}
			}
		}

		impl stream::Responder<#stream_type> for #name {
			fn receive(connection: std::sync::Weak<Connection>, stream: #stream_type) -> stream::Result<()> {
				let arc_conn = Connection::upgrade(&connection)?;
				arc_conn.clone().spawn(async move {
					let stream_impl = Self::new(connection, stream);
					stream_impl.#callback().await?;
					Ok(())
				});
				Ok(())
			}
		}
	};
	gen.into()
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
