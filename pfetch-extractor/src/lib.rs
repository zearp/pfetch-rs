use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;

#[proc_macro]
pub fn parse_logos(_input: TokenStream) -> TokenStream {
    let raw_logos = include_str!("../logos.sh");
    let raw_logos = raw_logos
        .split_once("in\n")
        .expect("Invalid logos.sh file")
        .1;
    let raw_logos = raw_logos
        .split_once("\nesac")
        .expect("Invalid logos.sh file")
        .0;

    let mut tux = None;
    let logos = raw_logos
        .split(";;\n")
        .filter_map(|raw_logo| {
            let (is_tux, logo) = parse_logo(raw_logo)?;
            if is_tux {
                tux = Some(logo.clone());
            }
            Some(logo)
        })
        .collect::<Vec<_>>();

    let tux = tux.unwrap();

    quote! { (#tux, [#(#logos),*]) }.into()
}

fn parse_logo(input: &str) -> Option<(bool, proc_macro2::TokenStream)> {
    let input = input.trim().replace('\t', "");
    if input.is_empty() {
        return None;
    }
    let regex = Regex::new(r"^\(?(.*)\)[\s\S]*read_ascii *(\d)? *(\d)?").unwrap();

    let groups = regex
        .captures(&input)
        .expect("Error while parsing logos.sh");

    let pattern = &groups[1];
    let primary_color = match groups.get(2) {
        Some(color) => color.as_str().parse::<u8>().unwrap(),
        None => 7,
    };
    let secondary_color = match groups.get(3) {
        Some(color) => color.as_str().parse::<u8>().unwrap(),
        None => (primary_color + 1) % 8,
    };

    let logo = input
        .split_once("EOF\n")
        .expect("Could not find start of logo")
        .1
        .split_once("\nEOF")
        .expect("Could not find end of logo")
        .0;

    let mut logo_parts = vec![];
    for logo_part in logo.split("${") {
        if let Some((new_color, rest)) = logo_part.split_once('}') {
            let new_color: u8 = match new_color {
                "c0" => 0,
                "c1" => 1,
                "c2" => 2,
                "c3" => 3,
                "c4" => 4,
                "c5" => 5,
                "c6" => 6,
                "c7" => 7,
                _ => panic!("Unknown color: {new_color}"),
            };
            let rest = rest.replace("\\\\", "\\");
            let rest = rest.replace("\\`", "`");
            let lines = rest.split('\n').collect::<Vec<_>>();
            let last_index = lines.len() - 1;
            for (index, line) in lines.into_iter().enumerate() {
                let mut line = line.to_owned();
                if index != last_index {
                    line += "\n";
                }
                logo_parts.push(quote! { (Color(#new_color), #line) });
            }
        } else if !logo_part.is_empty() {
            let logo_part = logo_part.replace("\\\\", "\\");
            logo_parts.push(quote! { (Color(9), #logo_part) });
        }
    }

    Some((
        pattern == "[Ll]inux*",
        quote! {
            Logo {
                primary_color: Color(#primary_color),
                secondary_color: Color(#secondary_color),
                pattern: #pattern,
                logo_parts: &[#(#logo_parts),*],
            }
        },
    ))
}