#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate termcolor;

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

fn scripts() -> Result<(), Box<dyn Error>> {
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
        let mut scripts: Vec<(String, String)> = package.scripts.into_iter().collect();
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
        write!(&mut buffer, " - {}", d)?;
    }

    writeln!(&mut buffer)?;

    for (command, instructions) in sorted_scripts {
        buffer.set_color(&script_name_color)?;
        writeln!(&mut buffer, "{}", command)?;

        buffer.set_color(&script_contents_color)?;
        writeln!(&mut buffer, "    {}", instructions)?;
    }

    bufwtr.print(&buffer)?;

    Ok(())
}

fn deps() -> Result<(), Box<dyn Error>> {
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

    fn fetch_deps(
        deps: &HashMap<String, String>
    ) -> Vec<(String, String, Option<String>)> {
        let mut deps: Vec<(String, String, Option<String>)> = deps
            .into_iter()
            .map(
                |(n, v)| -> Option<(String, String, Option<String>)> {
                    let package =
                        read_package::<DepPackage>(&format!("node_modules/{}/package.json", n));

                    match package {
                        Ok(package) => Some((package.name, v.to_string(), package.description)),
                        Err(e) => {
                            println!("error reading node_modules/{}/package.json: {}", n, e);
                            None
                        },
                    }
                },
            )
            .filter_map(|x| x )
            .collect();
        deps.sort_unstable_by(|(a, _, _), (b, _, _)| a.cmp(b));
        deps
    }

    let deps = fetch_deps(&package.dependencies);
    let dev_deps = fetch_deps(&package.dev_dependencies);

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
        write!(&mut buffer, " - {}", d)?;
    }

    writeln!(&mut buffer)?;

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
        Scripts,
        Deps,
    }

    let action = {
        let mut args = std::env::args();
        match args.nth(1) {
            Some(ref p) if p == "-d" => Action::Deps,
            Some(ref p) => {
                println!("unknown argument \"{}\"", p);
                return;
            }
            None => Action::Scripts,
        }
    };

    let res = match action {
        Action::Scripts => scripts(),
        Action::Deps => deps(),
    };

    if let Err(err) = res {
        println!("error occured: {}", err);
    }
}
