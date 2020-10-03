#![recursion_limit="128"]

extern crate syn;
#[macro_use] extern crate quote;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // Parse the string representation
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    // Build the impl
    generate_impl(&ast)
}

fn generate_impl(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields_vertex_attrib_pointer = generate_vertex_attrib_pointer_calls(&ast.data);

    let expanded = quote!{
        impl #ident #generics #where_clause {
            #[allow(unused_variables)]
            pub fn vertex_attrib_pointers(gl: &::gl::Gl) {
                let stride = ::std::mem::size_of::<Self>();
                let offset = 0;

                #(#fields_vertex_attrib_pointer)*
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn generate_vertex_attrib_pointer_calls(data: &syn::Data) -> Vec<proc_macro2::TokenStream> {
    match &data {
        &syn::Data::Enum(_) => panic!("VertexAttribPointers can not be implemented for enums"),
        &syn::Data::Union(_) => panic!("VertexAttribPointers can not be implemented for unions"),
        &syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unit, .. }) => {
            panic!("VertexAttribPointers can not be implemented for unit structs")
        },
        &syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(..), .. }) => {
            panic!("VertexAttribPointers can not be implemented for tuple structs")
        },
        &syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            fields.iter()
                .map(generate_struct_field_vertex_attrib_pointer_call)
                .collect()
        },
    }
}

fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> proc_macro2::TokenStream {
    // We are sure that the field has a name
    let field_name = field.ident.as_ref().unwrap().to_string();
    let location_attr = field.attrs
        .iter()
        .filter(|a| a.path.get_ident() == Some(&syn::Ident::new("location", proc_macro2::Span::call_site())))
        .next()
        .unwrap_or_else(|| panic!(
            "Field {:?} is missing #[location = x] attribute", field_name
        ));
    let location_value_literal = location_attr.tokens.to_owned().into_iter().find_map(|x| { 
        match x {
            quote::__private::TokenTree::Literal(literal) => Some(literal),
            _ => None,
        }
    }).unwrap();

    let field_ty = &field.ty;
    quote! {
        let location = #location_value_literal;
        unsafe {
            #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
        }
        let offset = offset + std::mem::size_of::<#field_ty>();
    }
}
