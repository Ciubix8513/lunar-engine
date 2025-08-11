use std::fmt::Write as _;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_attribute]
#[allow(clippy::missing_panics_doc)]
pub fn gen_swizzle(_: TokenStream, item: TokenStream) -> TokenStream {
    //Get struct name, generate a new trait from that -> NAMESwizzles {}
    //Get struct compoenent names and generate the variations of them
    //
    //N = Num fields in the sturct
    //total swizzles = N^2 + N^3 + N^4
    let i = item.clone();
    let input = parse_macro_input!(i as DeriveInput);

    let s = match input.data {
        syn::Data::Struct(data_struct) => data_struct,
        syn::Data::Union(_) | syn::Data::Enum(_) => {
            return "compile_error!(\"Must be a struct\")".parse().unwrap();
        }
    };

    let fields = match s.fields {
        syn::Fields::Named(fields_named) => fields_named.named,
        syn::Fields::Unnamed(_) | syn::Fields::Unit => {
            return "compile_error!(\"Must have named fields\")"
                .parse()
                .unwrap();
        }
    }
    .iter()
    .map(|i| i.ident.clone().unwrap().to_string())
    .collect::<Vec<_>>();

    let name = input.ident.to_string();
    let trait_name = format!("{name}Swizzles");

    let mut func_names = Vec::new();
    let mut return_types = Vec::new();
    let mut bodies = Vec::new();

    //Mildly scuffed, but eh
    for dimensions in 2..=4 {
        for x in 0..fields.len() {
            for y in 0..fields.len() {
                if dimensions >= 3 {
                    for z in 0..fields.len() {
                        if dimensions >= 4 {
                            for w in 0..fields.len() {
                                // 4 dim
                                func_names.push(format!(
                                    "{}{}{}{}",
                                    fields[x], fields[y], fields[z], fields[w]
                                ));

                                let ret = if fields.len() == dimensions {
                                    "Self"
                                } else {
                                    "Vec4"
                                };

                                return_types.push(ret);

                                bodies.push(format!(
                                    "{ret} {{x:self.{}, y:self.{},z:self.{}, w:self.{} }}",
                                    fields[x], fields[y], fields[z], fields[w],
                                ));
                            }
                        } else {
                            //3 dim
                            func_names.push(format!("{}{}{}", fields[x], fields[y], fields[z]));

                            let ret = if fields.len() == dimensions {
                                "Self"
                            } else {
                                "Vec3"
                            };

                            return_types.push(ret);

                            bodies.push(format!(
                                "{ret} {{x:self.{}, y:self.{},z:self.{} }}",
                                fields[x], fields[y], fields[z],
                            ));
                        }
                    }
                } else {
                    //2 dim
                    func_names.push(format!("{}{}", fields[x], fields[y]));

                    let ret = if fields.len() == dimensions {
                        "Self"
                    } else {
                        "Vec2"
                    };

                    return_types.push(ret);

                    bodies.push(format!(
                        "{ret} {{x:self.{}, y:self.{} }}",
                        fields[x], fields[y],
                    ));
                }
            }
        }
    }

    //For easer handling
    let mut trait_def = format!("///Swizzle functions for {name}\npub trait {trait_name} {{\n");
    let mut trait_impl = format!("impl {trait_name} for {name} {{");

    for (name, ret, body) in func_names
        .iter()
        .zip(return_types.iter())
        .zip(bodies)
        .map(|((x, y), z)| (x, y, z))
    {
        _ = write!(trait_def, "fn {name}(self) -> {ret};\n");
        _ = write!(trait_impl, "fn {name}(self) -> {ret} {{ {body} }}");
    }

    trait_def.push('}');
    trait_impl.push('}');

    let mut o = item;

    o.extend(trait_def.parse::<TokenStream>().unwrap());
    o.extend(trait_impl.parse::<TokenStream>().unwrap());

    o
}
