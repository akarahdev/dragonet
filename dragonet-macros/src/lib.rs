mod server;
mod client;

use proc_macro::{TokenStream};
use quote::quote;
use crate::client::client_impl;
use crate::server::server_impl;

#[proc_macro_attribute]
pub fn client(attr: TokenStream, item: TokenStream) -> TokenStream {
    client_impl(item.into()).into()
}

#[proc_macro_attribute]
pub fn server(attr: TokenStream, item: TokenStream) -> TokenStream {
    server_impl(item.into()).into()
}