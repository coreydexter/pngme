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
        decode           Read a message from a specified PNG file
        encode           Add a message to a specified PNG file
        help             Prints this message or the help of the given subcommand(s)
        identify-text    Identify the chunks which have pure text in them
        print            Display some information about the PNG and it's chunks
        remove           Remove a message from a specified PNG file

# Examples

## Encoding

    > target\release\pngme.exe encode examples/image.png teSt "Woah dude does this actually work?"

## Decoding 

    > target\release\pngme.exe decode examples/image.png teSt
    Woah dude does this actually work?

## Printing

    > target\release\pngme.exe print examples/image.png
    There are 23 chunks within this png
    0 - Chunk: 
        Length: 13
        Type:   IHDR
        Data:   13 bytes
        Crc:    2463534396

    1 - Chunk:
        Length: 1
        Type:   sRGB
        Data:   1 bytes
        Crc:    2932743401

    ...
    
    22 - Chunk:
        Length: 0
        Type:   IEND
        Data:   0 bytes
        Crc:    2923585666



## Identifying text

Produces lines of the format `{chunk index} - {chunk type} - {chunk message as UTF-8 string}`

    > target\release\pngme.exe identify-text examples/image.png
    22 - teSt - Woah dude does this actually work?


## Removing

    > target\release\pngme.exe remove examples/image.png teSt
    // Show that the message is no longer present
    > target\release\pngme.exe decode examples/image.png teSt
