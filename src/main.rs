#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate regex;
extern crate termcolor;

use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

fn color(c: Color) -> ColorSpec {
    let mut color = ColorSpec::new();
    color.set_fg(Some(c));
    color
}

fn read_package<T>(path: &str) -> Result<T, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    use std::fs::File;
    use std::io::BufReader;
    let file = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(file)?)
}

fn scripts(pattern: &Option<String>) -> Result<(), Box<dyn Error>> {
    #[derive(Deserialize)]
    struct Package {
        name: String,
        version: String,
        #[serde(default)]
        scripts: HashMap<String, String>,
        description: Option<String>,
    }

    let package = read_package::<Package>("package.json")?;

    let sorted_scripts = {
        let mut scripts: Vec<(String, String)> = match pattern {
            Some(pattern) => {
                let pattern = Regex::new(&pattern)?;
                package
                    .scripts
                    .into_iter()
                    .filter(|(s, _)| pattern.is_match(s))
                    .collect()
            }
            None => package.scripts.into_iter().collect(),
        };
        scripts.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        scripts
    };

    let mut name_color = color(Color::White);
    name_color.set_bold(true);
    let description_color = color(Color::White);
    let script_name_color = color(Color::Cyan);
    let script_contents_color = color(Color::White);

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buffer = bufwtr.buffer();

    buffer.set_color(&name_color)?;
    write!(&mut buffer, "{} {}", package.name, package.version)?;

    if let Some(d) = package.description {
        buffer.set_color(&description_color)?;
        write!(&mut buffer, " - {}", d);
    }

    writeln!(&mut buffer);

    for (command, instructions) in sorted_scripts {
        buffer.set_color(&script_name_color)?;
        writeln!(&mut buffer, "{}", command)?;

        buffer.set_color(&script_contents_color)?;
        writeln!(&mut buffer, "    {}", instructions)?;
    }

    bufwtr.print(&buffer)?;

    Ok(())
}

fn deps(pattern: &Option<String>) -> Result<(), Box<dyn Error>> {
    #[derive(Deserialize)]
    struct RootPackage {
        name: String,
        version: String,
        #[serde(default)]
        dependencies: HashMap<String, String>,
        #[serde(default, rename = "devDependencies")]
        dev_dependencies: HashMap<String, String>,
        description: Option<String>,
    }

    #[derive(Deserialize)]
    struct DepPackage {
        name: String,
        description: Option<String>,
    }

    let package = read_package::<RootPackage>("package.json")?;

    let search = match &pattern {
        Some(p) => Some(Regex::new(p)?),
        None => None,
    };

    fn fetch_deps(
        deps: &HashMap<String, String>,
        search: &Option<Regex>,
    ) -> Vec<(String, String, Option<String>)> {
        let mut deps: Vec<(String, String, Option<String>)> = deps
            .into_iter()
            .filter(|(n, _)| {
                if let Some(p) = &search {
                    return p.is_match(n);
                };
                true
            })
            .map(
                |(n, v)| -> Result<(String, String, Option<String>), Box<dyn Error>> {
                    let package =
                        read_package::<DepPackage>(&format!("node_modules/{}/package.json", n))?;

                    Ok((package.name, v.to_string(), package.description))
                },
            )
            .filter_map(|x| x.ok())
            .collect();
        deps.sort_unstable_by(|(a, _, _), (b, _, _)| a.cmp(b));
        deps
    }

    let deps = fetch_deps(&package.dependencies, &search);
    let dev_deps = fetch_deps(&package.dev_dependencies, &search);

    let mut name_color = color(Color::White);
    name_color.set_bold(true);
    let description_color = color(Color::White);
    let grey = color(Color::Rgb(100, 100, 100));
    let cyan = color(Color::Cyan);
    let white = color(Color::White);
    let green = color(Color::Green);

    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(&name_color)?;

    buffer.set_color(&name_color)?;
    write!(&mut buffer, "{} {}", package.name, package.version)?;

    if let Some(d) = package.description {
        buffer.set_color(&description_color)?;
        write!(&mut buffer, " - {}", d);
    }

    writeln!(&mut buffer);

    buffer.set_color(&green)?;
    writeln!(&mut buffer, "Depdencies: {}", deps.len())?;
    for (name, version, description) in deps {
        buffer.set_color(&cyan)?;
        write!(&mut buffer, "{}", name)?;
        buffer.set_color(&grey)?;
        writeln!(&mut buffer, " {}", version)?;
        if let Some(description) = description {
            buffer.set_color(&white)?;
            writeln!(&mut buffer, "    {}", description)?;
        }
    }
    buffer.set_color(&green)?;
    writeln!(&mut buffer, "Dev Depdencies: {}", dev_deps.len())?;
    for (name, version, description) in dev_deps {
        buffer.set_color(&cyan)?;
        write!(&mut buffer, "{}", name)?;
        buffer.set_color(&grey)?;
        writeln!(&mut buffer, " {}", version)?;
        if let Some(description) = description {
            buffer.set_color(&white)?;
            writeln!(&mut buffer, "    {}", description)?;
        }
    }
    bufwtr.print(&buffer)?;

    Ok(())
}

fn main() {
    enum Action {
        Scripts(Option<String>),
        Deps(Option<String>),
    }

    let action = {
        let mut args = std::env::args();
        match args.nth(1) {
            Some(ref p) if p == "-d" => Action::Deps(args.nth(0)),
            p => Action::Scripts(p),
        }
    };

    let res = match action {
        Action::Scripts(p) => scripts(&p),
        Action::Deps(p) => deps(&p),
    };

    if let Err(err) = res {
        println!("error occured: {}", err);
    }
}
