# unin - by notchapplez
<img src="https://hackatime.hackclub.com/api/v1/badge/U08T8QD95U1/notchapplez/unin">

### Now you may ask – what is unin?
#### → And I have the answer: 
##### unin is a simple, yet powerful, universal installer. I know you hate remembering all the commands for compiling projects in all the different languages. 
###### So I made this. I thought of this in my sleep, and I'm glad I did. 

#### unin has also a feature to self-update without having to manually copy the files and compile the code.
#### unin also moves all the release files to /usr/local/bin so they don't conflict with other executables in /usr/bin.


### Note: unin is only supported on Linux, and is only tested on Arch Linux x64_86. Report bugs in the 'Issues' tab.

## As of May 2026, unin supports the following languages/build systems:
- Rust
- CMake
- Make
- Go
- Zig
- Haskell (hella buggy)
- gleam
- configure
- Meson
## Syntax:
| Command                                                                        | Description                                                                                |
|--------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------|
| unin --setup <full(all the languages),rust,cmake,make,go,zig,swift,haskell,d,> | Setup languages system-wide                                                                |
| unin <"path"> (--noinstall)                                                    | Compile project at the given path, noinstall only compiles the binaries, doesn't move them |
| unin --clean  <"path">                                                         | Clean the artefacts built                                                                  |                                                                                          
| unin --uninstall <'package'>                                                   | Uninstalls a package (if it was added to the registry)                                     |
| unin --queue-registry                                                          | Shows packages that are registered in the registry                                         |
| unin --help                                                                    | Shows the help menu                                                                        |                                                                                         
## Installation:
### There are three ways to install unin:
#### 1. Using cargo. Run "cargo install unin-bin" in your terminal. 
#### 2. Precompiled binaries. Head to the "releases" page and download the latest release. Open a terminal and run "chmod +x unin" to make the file executable. After that, copy the file to /usr/local/bin usint the command "sudo cp <path_to_unin_executable> /usr/local/bin".
#### 3. Compile from source code yourself. This requires rust to be installed. Clone the repository with "git clone https://github.com/notchapplez/unin". Change the directory to the cloned repository and run 'cargo run --release'. This will set up unin on your system. Ensure /usr/local/bin is set in PATH.

My beautiful commit graph (credits to hackatime-heatmap.shymike.dev)
<a href="https://heatmap.shymike.dev?id=notchapplez&timezone=Europe%2FAmsterdam&labels=true&standalone=true" title="Click to view detailed data for each day!">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://heatmap.shymike.dev?id=notchapplez&timezone=Europe%2FAmsterdam&labels=true&theme=catppuccin_dark">
        <img alt="Hackatime activity heatmap" src="https://heatmap.shymike.dev?id=notchapplez&timezone=Europe%2FAmsterdam&labels=true&theme=catppuccin_light">
    </picture>
</a>

## Tech Stack
#### -Rust (including .toml)
#### -Main packages simplifying my coding experience:
##### • serde-json
##### • colored
##### • dialoguer
##### • duct

### What I learned
#### I learned how to use BufReaders, how to make my code a bit cleaner, how to implement OOP in my program, that not every build system likes me, and that I love Rust.
