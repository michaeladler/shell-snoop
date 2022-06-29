use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::process::{self, Stdio};

use anyhow::Result;
use caps::{self, CapSet, Capability};
use libproc::libproc::proc_pid;
use rev_lines::RevLines;

fn main() -> Result<()> {
    caps::raise(None, CapSet::Inheritable, Capability::CAP_SYS_PTRACE)?;
    caps::raise(None, CapSet::Ambient, Capability::CAP_SYS_PTRACE)?;

    let args = env::args();
    for pid in args.skip(1) {
        let pid: i32 = pid.parse::<i32>()?;

        let contents = fs::read_to_string(format!("/proc/{}/task/{}/children", pid, pid))?;
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
            let dumpfile = format!("/tmp/shell-history-{}.txt", pid);
            let mut cmd = process::Command::new("gdb")
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
                .spawn()?;
            cmd.wait()?;

            match File::open(&dumpfile) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let mut rev_lines = RevLines::new(reader)?;
                    if let Some(last_line) = rev_lines.next() {
                        let cmd = match shell {
                            Shell::Zsh => last_line
                                .split(';')
                                .nth(1)
                                .expect("Unable to determine command"),
                            Shell::Bash => &last_line,
                        };
                        println!("{}", cmd);
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
    }

    Ok(())
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
