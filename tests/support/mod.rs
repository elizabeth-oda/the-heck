//! Isolated interactive shells for the terminal integration tests.
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::{symlink, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tempfile::TempDir;

pub const BINARY: &str = env!("CARGO_BIN_EXE_heck");
const CAPTURE: &str = "\x18\x14";
const PROMPT: &str = "__HECK_TEST_PROMPT__";
const TIMEOUT: Duration = Duration::from_secs(5);
// openpty has no CLOEXEC flag. Serialize descriptor creation and spawning so
// concurrent test children cannot inherit a descriptor before we mark it.
static SPAWN_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
}

pub const SHELLS: [Shell; 2] = [Shell::Bash, Shell::Zsh];

impl Shell {
    pub fn name(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
        }
    }

    pub fn choose<'a>(self, bash: &'a str, zsh: &'a str) -> &'a str {
        match self {
            Self::Bash => bash,
            Self::Zsh => zsh,
        }
    }
}

pub fn run(command: &mut Command) -> Output {
    let _guard = SPAWN_LOCK.lock().unwrap();
    command.output().unwrap()
}

pub fn executable(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

fn shell_path(shell: Shell) -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join(shell.name()))
        .find(|path| {
            path.is_file()
                && fs::metadata(path)
                    .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
        })
        .unwrap_or_else(|| {
            panic!(
                "{} is required for terminal tests; install Bash and Zsh before cargo test",
                shell.name()
            )
        })
}

/// Allocate a terminal with owned descriptors.
fn terminal() -> io::Result<(File, File)> {
    let mut master = -1;
    let mut slave = -1;
    let mut size = libc::winsize {
        ws_row: 30,
        ws_col: 120,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: all output pointers and size are valid for this call. Null name
    // and termios request the platform defaults.
    let status = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(size),
        )
    };
    if status == -1 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a successful openpty returns two new descriptors, owned here.
    let (master, slave) = unsafe { (File::from_raw_fd(master), File::from_raw_fd(slave)) };
    for file in [&master, &slave] {
        // SAFETY: each file owns a valid descriptor; F_SETFD takes an integer.
        if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } == -1 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok((master, slave))
}

pub struct Session {
    shell: Shell,
    child: Child,
    master: File,
    transcript: Vec<u8>,
    // Drop the terminal before cleaning up the session's files.
    pub root: TempDir,
}

impl Session {
    pub fn new(shell: Shell) -> Self {
        Self::configured(shell, "emacs", "")
    }

    pub fn configured(shell: Shell, mode: &str, setup: &str) -> Self {
        let program = shell_path(shell);
        let root = tempfile::Builder::new()
            .prefix("heck-pty-")
            .tempdir()
            .unwrap();
        let bin = root.path().join("bin");
        fs::create_dir(&bin).unwrap();
        symlink(BINARY, bin.join("heck")).unwrap();
        // A NUL-delimited argument log, prefixed by argc for each invocation,
        // preserves empty arguments and newlines without another test runtime.
        let recorder = r#"#!/bin/sh
# Metadata queries must not appear as command submissions in the argument log.
case "$*" in
    *"config --null --name-only --get-regexp"*) exit 1 ;;
    "alias list") printf 'co: pr checkout\n'; exit 0 ;;
    "extension list") printf 'gh stack\tgithub/gh-stack\tv1\n'; exit 0 ;;
esac
{
    printf '%s\0' "$#"
    for arg do printf '%s\0' "$arg"; done
} >> "$HOME/runs"
case "$1" in stat|stats|bulid) exit 1;; esac
"#;

        for name in ["git", "cargo", "gh"] {
            executable(&bin.join(name), recorder);
        }
        let mut rc = format!(
            "PS1='{PROMPT} '\nHISTFILE=/dev/null\nHISTSIZE=100\n{setup}\nsource <(command heck init {})\n",
            shell.name()
        );
        // Test-only binding to inspect the editable prompt without submitting it.
        rc.push_str(shell.choose(
            r#"
__capture() { printf '%s\0' "$READLINE_LINE" > "$HOME/buffer"; }
for m in emacs-standard vi-insert vi-command; do bind -m "$m" -x '"\C-x\C-t":__capture'; done
"#,
            r#"
__capture() { printf '%s\0' "$BUFFER" > "$HOME/buffer"; }
zle -N __capture
for m in emacs viins vicmd; do bindkey -M "$m" '^X^T' __capture; done
"#,
        ));
        rc.push_str(match (shell, mode) {
            (Shell::Bash, "vi") => "set -o vi\n",
            (Shell::Bash, _) => "set -o emacs\n",
            (Shell::Zsh, "vi") => "bindkey -v\n",
            (Shell::Zsh, _) => "bindkey -e\n",
        });
        let rc_path = root.path().join(shell.choose("bashrc", ".zshrc"));
        fs::write(&rc_path, rc).unwrap();

        let mut command = Command::new(program);
        if shell == Shell::Bash {
            command
                .args(["--noprofile", "--rcfile"])
                .arg(&rc_path)
                .arg("-i");
        } else {
            command.args(["-d", "-i"]);
        }
        command
            .current_dir(root.path())
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
            .env("HOME", root.path())
            .env("ZDOTDIR", root.path())
            .env("TERM", "xterm-256color")
            .env("LC_ALL", "C.UTF-8");
        let (master, child) = {
            let _guard = SPAWN_LOCK.lock().unwrap();
            let (master, slave) = terminal().unwrap();
            command
                .stdin(Stdio::from(slave.try_clone().unwrap()))
                .stdout(Stdio::from(slave.try_clone().unwrap()))
                .stderr(Stdio::from(slave));
            // SAFETY: the child hook only calls async-signal-safe Unix routines.
            // Command installs the slave on stdin before invoking this hook.
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1
                        || libc::ioctl(libc::STDIN_FILENO, libc::TIOCSCTTY, 0) == -1
                    {
                        return Err(io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
            (master, command.spawn().unwrap())
        };
        let mut session = Self {
            root,
            shell,
            child,
            master,
            transcript: Vec::new(),
        };
        session.prompt();
        session
    }

    fn fail(&self, message: &str) -> ! {
        panic!(
            "{}: {message}\nTerminal transcript:\n{}",
            self.shell.name(),
            String::from_utf8_lossy(&self.transcript)
        );
    }

    pub fn send(&mut self, data: &str) {
        if let Err(error) = self.master.write_all(data.as_bytes()) {
            self.fail(&format!("cannot write to terminal: {error}"));
        }
    }

    fn read(&mut self) -> bool {
        let mut descriptor = libc::pollfd {
            fd: self.master.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: descriptor is a valid single-element pollfd array.
        let ready = unsafe { libc::poll(&mut descriptor, 1, 50) };
        if ready == 0 {
            return false;
        }
        if ready == -1 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                return false;
            }
            self.fail(&format!("cannot poll terminal: {error}"));
        }
        let mut bytes = [0; 4096];
        match self.master.read(&mut bytes) {
            Ok(0) => self.fail("shell exited"),
            Ok(count) => {
                self.transcript.extend_from_slice(&bytes[..count]);
                true
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => false,
            Err(error) => self.fail(&format!("cannot read terminal: {error}")),
        }
    }

    pub fn drain(&mut self) {
        let deadline = Instant::now() + TIMEOUT;
        while self.read() {
            if Instant::now() >= deadline {
                self.fail("terminal did not settle");
            }
        }
    }

    pub fn wait_for(&mut self, needle: &str) {
        let start = self.transcript.len();
        let deadline = Instant::now() + TIMEOUT;
        while Instant::now() < deadline {
            self.read();
            if self.transcript[start..]
                .windows(needle.len())
                .any(|window| window == needle.as_bytes())
            {
                return;
            }
        }
        self.fail(&format!("timed out waiting for {needle:?}"));
    }

    pub fn prompt(&mut self) -> String {
        let start = self.transcript.len();
        self.wait_for(PROMPT);
        self.drain();
        String::from_utf8_lossy(&self.transcript[start..]).into_owned()
    }

    pub fn command(&mut self, command: &str) -> String {
        self.send(command);
        self.send("\r");
        self.prompt()
    }

    pub fn capture(&mut self) -> String {
        let path = self.root.path().join("buffer");
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => panic!("{error}"),
        }
        self.send(CAPTURE);
        let deadline = Instant::now() + TIMEOUT;
        while Instant::now() < deadline {
            if let Ok(bytes) = fs::read(&path) {
                if bytes.last() == Some(&0) {
                    let buffer = String::from_utf8(bytes[..bytes.len() - 1].to_vec()).unwrap();
                    self.drain();
                    return buffer;
                }
            }
            self.read();
        }
        self.fail("buffer capture timed out");
    }

    pub fn picker(&mut self) {
        self.wait_for("Esc/Ctrl-C: cancel");
    }

    pub fn select_for_editing(&mut self) {
        self.send("\r");
        self.wait_for(self.shell.choose("heck> ", PROMPT));
        self.drain();
    }

    pub fn runs(&self) -> Vec<Vec<String>> {
        let path = self.root.path().join("runs");
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Vec::new(),
            Err(error) => panic!("{error}"),
        };
        assert_eq!(bytes.last(), Some(&0), "incomplete command recording");
        let mut fields = bytes[..bytes.len() - 1].split(|byte| *byte == 0);
        let mut runs = Vec::new();
        while let Some(count) = fields.next() {
            let count: usize = std::str::from_utf8(count).unwrap().parse().unwrap();
            runs.push(
                (0..count)
                    .map(|_| String::from_utf8(fields.next().unwrap().to_vec()).unwrap())
                    .collect(),
            );
        }
        runs
    }

    pub fn read_file(&self, name: &str) -> String {
        fs::read_to_string(self.root.path().join(name)).unwrap()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Closing the master also hangs up any remaining foreground command.
    }
}
