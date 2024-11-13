use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn};

pub fn server_impl(item: TokenStream) -> TokenStream {
    println!("{}", item.to_string());

    let function = parse2::<ItemFn>(item.clone()).unwrap();
    let name = function.sig.ident.clone();

    assert_ne!(name.to_string(), "main");

    quote! {
        #item

        pub fn main() {
            let mut _srv: Server<ProtocolState, Packets> = Server::new();
            #name(&mut _srv);
            _srv.event_loop();
        }
    }
}