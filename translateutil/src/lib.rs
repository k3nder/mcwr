extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use std::{env, fs};
use syn::{parse_macro_input, LitStr};
use toml::Value;

#[proc_macro]
pub fn translate(input: TokenStream) -> TokenStream {
    // Parseamos el input (esperamos una cadena literal)
    let input = parse_macro_input!(input as LitStr);

    #[cfg(feature = "en")]
    let lang = "en";

    #[cfg(feature = "es")]
    let lang = "es";

    #[cfg(feature = "fr")]
    let lang = "fr";

    #[cfg(feature = "ru")]
    let lang = "ru";

    // Intentamos obtener la ruta base usando el directorio de compilación
    // CARGO_MANIFEST_DIR apunta al directorio donde está el Cargo.toml
    let base_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            // Fallback a directorio actual
            ".".to_string()
        }
    };

    // Construimos la ruta completa
    let target_path = Path::new(&base_dir).join(format!("translations/{}.toml", lang));

    // Leemos el contenido del archivo
    let content = match fs::read_to_string(&target_path) {
        Ok(content) => content,
        Err(e) => {
            // En caso de error, generamos un error de compilación
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Error reading file '{}': {}", target_path.display(), e),
            )
            .to_compile_error()
            .into();
        }
    };

    let parsed_toml = content.parse::<toml::Value>().unwrap();
    let content = get_nested(&parsed_toml, &input.value())
        .expect(format!("Error translating key {} not found", input.value()).as_str())
        .as_str()
        .expect(
            format!(
                "Expected a string value on translation key {}",
                input.value()
            )
            .as_str(),
        );
    // Generamos el código que será la cadena literal con el contenido
    let content_literal = proc_macro2::Literal::string(&content);
    let output = quote! { #content_literal };

    // Convertimos a TokenStream y retornamos
    output.into()
}

fn get_nested<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    key.split('.').fold(Some(value), |acc, k| acc?.get(k))
}
