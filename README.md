# Purpose

Following along with the [pngme_book](https://picklenerd.github.io/pngme_book/introduction.html) tutorial to learn and get familar with the Rust language.

# What does it do?

pngme is a command line application that allows adding, removing, or viewing specific chunks in a PNG file. It's main purpose it to add secret messages to your images! 

# Getting started

Build the executable using cargo, aka

    > cargo build --release

Run the release with

    > cargo run --release -- *params here*

Or directly like

    > target\release\pngme.exe *params here*

To see a list of the parameters, use the `--help` command, eg

    > target\release\pngme.exe --help
    pngme 0.1.0
     
    USAGE:  
       pngme.exe <SUBCOMMAND>
    
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    SUBCOMMANDS:
        decode
        encode
        help      Prints this message or the help of the given subcommand(s)
        remove    

# Examples

## Encoding

    > target\release\pngme.exe encode examples/image.png teSt "Woah dude does this actually work?"

## Decoding 

    > target\release\pngme.exe decode examples/image.png teSt
    Woah dude does this actually work?

## Removing

    > target\release\pngme.exe remove examples/image.png teSt
    // Show that the message is no longer present
    > target\release\pngme.exe decode examples/image.png teSt
