use serde::{de::DeserializeOwned, Deserialize};
use structopt::StructOpt;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

fn color(c: Color) -> ColorSpec {
    let mut color = ColorSpec::new();
    color.set_fg(Some(c));
    color
}

fn read_package<T>(path: &Path) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let file = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(file)?)
}

fn scripts(path: PathBuf) -> Result<(), Box<dyn Error>> {
    #[derive(Deserialize)]
    struct Package {
        name: String,
        version: String,
        #[serde(default)]
        scripts: HashMap<String, String>,
        description: Option<String>,
    }

    let package = read_package::<Package>(&path.join("package.json"))?;

    let sorted_scripts = {
        let mut scripts: Vec<(String, String)> = package.scripts.into_iter().collect();
        scripts.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        scripts
    };

    let name_color = color(Color::White);
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

fn deps(path: PathBuf) -> Result<(), Box<dyn Error>> {
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

    let package = read_package::<RootPackage>(&path.join("package.json"))?;

    fn fetch_deps(
        path: &Path,
        deps: &HashMap<String, String>,
    ) -> Vec<(String, String, Option<String>)> {
        let mut deps: Vec<(String, String, Option<String>)> = deps
            .iter()
            .map(|(n, v)| -> Option<(String, String, Option<String>)> {
                let package = read_package::<DepPackage>(
                    &path.join(format!("node_modules/{}/package.json", n)),
                );

                match package {
                    Ok(package) => Some((package.name, v.to_string(), package.description)),
                    Err(e) => {
                        eprintln!("error reading node_modules/{}/package.json: {}", n, e);
                        None
                    }
                }
            })
            .filter_map(|x| x)
            .collect();
        deps.sort_unstable_by(|(a, _, _), (b, _, _)| a.cmp(b));
        deps
    }

    let deps = fetch_deps(&path, &package.dependencies);
    let dev_deps = fetch_deps(&path, &package.dev_dependencies);

    let name_color = color(Color::White);
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
    write!(&mut buffer, "Depdencies: ")?;
    buffer.set_color(&white)?;
    writeln!(&mut buffer, "{}", deps.len())?;
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
    write!(&mut buffer, "Dev Depdencies: ")?;
    buffer.set_color(&white)?;
    writeln!(&mut buffer, "{}", dev_deps.len())?;
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

/// Utility for quickly displaying package.json information
#[derive(StructOpt, Debug)]
#[structopt(name = "pj")]
struct Opt {
    /// List dependencies instead of scripts
    #[structopt(short, long)]
    dependencies: bool,

    /// The project to examine. Current directory will be used by default
    #[structopt(name = "PATH", parse(from_os_str))]
    path: Option<PathBuf>,
}

fn main() {
    let opt: Opt = Opt::from_args();

    let path = opt.path.unwrap_or_else(|| std::env::current_dir().unwrap());

    let result = if opt.dependencies {
        deps(path)
    } else {
        scripts(path)
    };

    if let Err(err) = result {
        eprintln!("error occured: {}", err);
        std::process::exit(1);
    }
}
