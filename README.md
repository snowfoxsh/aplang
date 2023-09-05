# Ap Lang
A language designed to aid students who are taking the AP Computer Science Principals exam. 

# Quick Start
If you are looking simply to install Ap Lang and not contribute installing with cargo from crates.io is recommended.
See section: [From Cargo](#from-cargo). Windows and macOS installers are a work in progress; until then you will
need [Rust](https://rust-lang.org).

## Building
To build ApLang you will need the latest stable version of rust as well as git. If you are building 
> If you do not have rust follow the instructions at [rustup.rs](https://rustup.rs). \
> If you do not have git follow the instructions at [git-scm.com](https://git-scm.com/downloads).

### From Cargo
To install from [crates.io](https://crates.io/crates/aplang) run: 

```bash
cargo install aplang
```
Please note this will automatically add the aplang binary to your path.

### From Source
To build locally, first clone the source code:
```bash
git clone https://github.com/snowfoxsh/aplang.git
```

Then compile the code:
```bash
cd aplang
cargo build --release
```

Run the project:
```bash
cargo run --release -- ./example.ap
```

Alternatively:
```bash
cd ./target/release
./aplang ./example.ap
```

#### Testing
Testing is as simple as:
```bash
cargo test --release --all
```

Testing individual modules can be done with:
```bash
cargo test --release parser::tests
```

## Accessibility
The goal with this project is accessibility first. It is understood that the vast majority of people who will 
use the interpreter are new to programming. Therefore, it is of the upmost import that installation is made simple and 
documentation is extensive. Linux is wonderful however windows support, macOS and Web support must be put first
simply because those are the systems that most new programmers will use.


## For Students
I made this project to make your life easier and to help YOU get a better grade on the AP Computer Science Principals Test. 
In return for my work I simply ask that you share this project with your teachers and fellow students. The more people
that know of it the better.


## Contributing 
I welcome all contributors with open arms. I will eventually make a guide for contributing but that will be done on
project v1.0. I apologize for my garbage commit messages in advance. If you have any questions feel free to add me on discord
at @dev_storm or contact me via email at [dev_storm@winux.com](mailto:dev_storm@example.com). I will make a discord server as well as matrix chat if 
this project gains traction.

### Feature Request
Please submit an issue ticket

### A Note
I chose rust for this project because cargo is an amazing build system. Rust can run anywhere! 

#### For Collage Board
Please bring awareness to the project! I would love for it to become officially supported however unlikely that would be.
Contact me!

#### Licence
This project is licensed under [GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html) because that is what is [recommended](https://creativecommons.org/about/program-areas/software/) by creative commons 
