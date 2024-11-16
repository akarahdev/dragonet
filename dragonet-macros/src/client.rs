use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn};

pub fn client_impl(item: TokenStream) -> TokenStream {
    println!("{}", item.to_string());

    let function = parse2::<ItemFn>(item.clone()).unwrap();
    let name = function.sig.ident.clone();

    assert_ne!(name.to_string(), "main");
    assert_eq!(function.sig.inputs.len(), 1);

    quote! {
        #item

        pub fn main() {
            let mut _client: Client<ProtocolState, Packets> = Client::new();
            #name(&mut _client);
            _client.event_loop();
        }
    }
}