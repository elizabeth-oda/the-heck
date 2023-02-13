# the-heck
A comamnd line corrector inspired by `thefuck` and built in Rust.

# What it does
the-heck is a tool for fixing your command line mistakes. 
For example: 
```
(personal) ➜  the_heck git:(main) git stats 
git: 'stats' is not a git command. See 'git --help'.

The most similar command is
        status

(personal) ➜  the_heck git:(main) heck
✔ Is this command correct? [Press Enter]  · status
Fix successful!
On branch main
Your branch is up to date with 'origin/main'.

nothing to commit, working tree clean

(personal) ➜  the_heck git:(main) 
```

Version `0.1.4` is very much a work-in-progress. It only supports a handful of commands and there are known bugs.

## Currently supported programs and arguments

`git` + `add .`, `branch`, `restore --staged .`, `restore .`, `status`

`cargo` + `build`, `clippy`, `fmt`, `install`, `run`, `test`

# Installation
Assuming that you have installed Rust and Cargo properly, simply run the following in your desired project directory:
```
cargo add the_heck
```
