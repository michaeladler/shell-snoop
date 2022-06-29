use std::fs::{self, File};
use std::io::BufReader;
use std::process::{self, Stdio};
use std::{env, fmt};

use caps::{self, CapSet, Capability};
use libproc::libproc::proc_pid;
use rev_lines::RevLines;

fn main() {
    if caps::raise(None, CapSet::Inheritable, Capability::CAP_SYS_PTRACE).is_err() {
        eprintln!("Failed to raise inheritable capability. Trying anyway.");
    }
    if caps::raise(None, CapSet::Ambient, Capability::CAP_SYS_PTRACE).is_err() {
        eprintln!("Failed to raise ambient capability. Trying anyway.");
    }

    let args = env::args();
    for pid in args.skip(1).flat_map(|pid| pid.parse::<i32>()) {
        let contents =
            fs::read_to_string(format!("/proc/{}/task/{}/children", pid, pid)).unwrap_or_default();
        if contents.is_empty() {
            // avoid starting gdb since it is an expensive operation
            eprintln!("skipping pid {} because it has not children", pid);
            continue;
        }

        let shell = match proc_pid::name(pid) {
            Ok(name) => match name.as_str() {
                "zsh" => Some(Shell::Zsh),
                "bash" => Some(Shell::Bash),
                _ => None,
            },
            _ => None,
        };

        if let Some(shell) = shell {
            let dumpfile = format!("/tmp/{}-history-{}.txt", shell, pid);

            match process::Command::new("gdb")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .arg("-n")
                .arg("-batch")
                .arg("--eval")
                .arg(format!("attach {}", pid))
                .arg("--eval")
                .arg(shell.dumpcmd(&dumpfile))
                .arg("--eval")
                .arg("detach")
                .arg("--eval")
                .arg("q")
                .spawn()
            {
                Ok(mut cmd) => {
                    if let Err(e) = cmd.wait() {
                        eprintln!("gdb failed with non-zero exit code: {}", e);
                    }

                    match File::open(&dumpfile) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            if let Ok(mut rev_lines) = RevLines::new(reader) {
                                if let Some(last_line) = rev_lines.next() {
                                    let cmd = match shell {
                                        Shell::Zsh => last_line.split(';').nth(1),
                                        Shell::Bash => Some(last_line.as_str()),
                                    };
                                    if let Some(cmd) = cmd {
                                        println!("{}", cmd);
                                    }
                                }
                            }

                            if let Err(e) = std::fs::remove_file(&dumpfile) {
                                eprintln!("Failed to remove dump file: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading {}: {}", dumpfile, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to spawn gdb: {}. Make sure 'gdb' is installed!", e);
                }
            }
        }
    }
}

enum Shell {
    Bash,
    Zsh,
}

impl Shell {
    pub fn dumpcmd(&self, dumpfile: &str) -> String {
        match self {
            Shell::Zsh => format!("call (void)savehistfile(\"{}\", 0, 0)", &dumpfile),
            Shell::Bash => format!("call (void)write_history(\"{}\")", &dumpfile),
        }
    }
}

impl fmt::Display for Shell {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Shell::Bash => "bash",
                Shell::Zsh => "zsh",
            }
        )
    }
}
