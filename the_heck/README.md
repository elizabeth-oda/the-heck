# the-heck
A comamnd line corrector inspired by `thefuck` and built in Rust.

# What it does
the-heck is a tool for fixing your command line mistakes. 
For example: 
```
the_heck git:(main) cargo ruu                                                                
error: no such subcommand: `ruu`

        Did you mean `run`?
the_heck git:(main) heck
✔ Is this command correct? [Press Enter]  · run
```

Version `0.1.0` is very much a work-in-progress. It only supports a handful of commands and there are known bugs.


# Installation
Assuming that you have installed Rust and Cargo properly, simply run the following in your desired project directory:
```
cargo add the_heck
```
