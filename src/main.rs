use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use walkdir::WalkDir;
use serde_json;
use std::path::Path;

#[derive(Deserialize)]

struct StrapiComponent {
    collectionName: String,
    info: Info,
    attributes: std::collections::HashMap<String, Attribute>,
}

#[derive(Deserialize)]
struct Info {
    displayName: String,
    icon: String,
}

#[derive(Deserialize)]
struct Attribute {
    #[serde(rename = "type")]
    attr_type: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    multiple: bool,
}

fn main() -> io::Result<()> {

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("How to use: {} <path_to_yor_strapi_components>", args[0]);
        std::process::exit(1);
    }

    let strapi_components_path = &args[1];

    let current_dir = std::env::current_dir().unwrap();

    let paths = WalkDir::new(strapi_components_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "json"));

    for path in paths {
        writeln!(io::stdout(), "Processing: {}", path.path().display())?;
        let parent = path.path().parent().unwrap().strip_prefix(strapi_components_path).unwrap();
        writeln!(io::stdout(), "Parent: {}", parent.display())?;
        let file = File::open(path.path()).unwrap();
        let component: StrapiComponent = serde_json::from_reader(file)?;
        let output_path = format!("{}/components/{}/{}.jsx", current_dir.display() ,parent.display(), capitalize_first_letter(&component.info.displayName));
        writeln!(io::stdout(), "Output: {}", output_path)?;
        if let Some(dir_path) = Path::new(&output_path).parent() {
            fs::create_dir_all(dir_path)?;
        }
        let mut output_file = File::create(&output_path).unwrap();
        writeln!(output_file, "{}", generate_react_component(&component))?;
    }

    writeln!(io::stdout(), "Done!")?;

    Ok(())
}

fn capitalize_first_letter(s: &str) -> String {
    s.char_indices()
     .next()
     .map(|(i, c)| c.to_uppercase().collect::<String>() + &s[i+1..])
     .unwrap_or_else(|| String::new())
}

fn generate_react_component(component: &StrapiComponent) -> String {
    let props = component.attributes.keys().map(|key| key.as_str()).collect::<Vec<&str>>().join(", ");

    let formated_props = format!("{{ {props} }}", props = props);

    let jsx = component
        .attributes
        .keys()
        .map(|key| format!("{{ {key} }}", key = key))
        .collect::<Vec<String>>()
        .join("\n      ");

    format!(
        "const {name} = ({props}) => {{
  return (
    <div>
      {jsx}
    </div>
  );
}};

export default {name};
",
        name = capitalize_first_letter(&component.info.displayName),
        props = formated_props,
        jsx = jsx
    )
}
