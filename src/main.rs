use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::process::{self, Stdio};

use anyhow::Result;
use caps::{self, CapSet, Capability};
use proc_status::{ProcRef, ProcStatus};
use rev_lines::RevLines;

fn main() -> Result<()> {
    caps::raise(None, CapSet::Inheritable, Capability::CAP_SYS_PTRACE)?;
    caps::raise(None, CapSet::Ambient, Capability::CAP_SYS_PTRACE)?;

    let args = env::args();
    for pid in args.skip(1) {
        let pid: usize = pid.parse::<usize>()?;

        let contents = fs::read_to_string(format!("/proc/{}/task/{}/children", pid, pid))?;
        if contents.is_empty() {
            // avoid starting gdb since it is an expensive operation
            eprintln!("skipping pid {} because it has not children", pid);
            continue;
        }

        let dumpfile = format!("/tmp/shell-history-{}.txt", pid);

        let status = ProcStatus::read_for(ProcRef::ProcId(pid))?;
        let is_zsh = matches!(status.value("Name"), Ok("zsh"));
        let dump_cmd = if is_zsh {
            format!("call (void)savehistfile(\"{}\", 0, 0)", &dumpfile)
        } else {
            format!("call (void)write_history(\"{}\")", &dumpfile)
        };

        let mut cmd = process::Command::new("gdb")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .arg("-n")
            .arg("-batch")
            .arg("--eval")
            .arg(format!("attach {}", pid))
            .arg("--eval")
            .arg(dump_cmd)
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
                    let cmd = if is_zsh {
                        last_line
                            .split(';')
                            .nth(1)
                            .expect("Unable to determine command")
                    } else {
                        &last_line
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

    Ok(())
}
