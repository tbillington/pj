pj
--

`pj` is a tool for quickly exploring JavaScript applications or libraries.

I created this tool because I found myself switching between lots of JS repos and figuring out what each of them is can be time consuming. My previous strategy was to `cat package.json`. While `cat` acomplished what I wanted, there was lots of noise, `package.json` files often contain a lot of information that isn't necessary for my purpose, along with being organised in different ways in every project.

My goals are:

- See the scripts/commands available
- See the dependencies

Seeing those pieces of information at a glance quickly gives me an idea of what the project is and how to use it.

### Installation

If you have [Rust](https://www.rust-lang.org/) installed you can run the below instructions to have `pj` installed to your path.

```
git clone https://www.github.com/tbillington/pj
cargo install --path pj
```

If you don't have rust, you can get the Rust installer [here](https://www.rust-lang.org/tools/install).

### Usage

```
pj 0.2.0
Utility for quickly displaying package.json information

USAGE:
    pj [FLAGS] [PATH]

FLAGS:
    -d, --dependencies    List dependencies instead of scripts
    -h, --help            Prints help information
    -V, --version         Prints version information

ARGS:
    <PATH>    The project to examine. Current directory will be used by default
```

There are two main functions currently supported by `pj`. The first and default option is to display the [npm scripts](https://docs.npmjs.com/files/package.json#scripts) available, and the second to display the [dependencies](https://docs.npmjs.com/files/package.json#dependencies).

#### Displaying scripts

Run `pj` in the root directory containing `package.json`.

![screen shot 2019-02-18 at 5 41 33 pm](https://user-images.githubusercontent.com/2771466/52987192-4ae31400-344e-11e9-84cf-87acf3ca3d36.png)

#### Display dependencies

Run `pj -d` in the root directory containing `package.json`.

![screen shot 2019-02-18 at 5 42 13 pm](https://user-images.githubusercontent.com/2771466/52987204-55051280-344e-11e9-97c4-78bacd915edd.png)
