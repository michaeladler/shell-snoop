[![build](https://github.com/michaeladler/shell-snoop/actions/workflows/build.yml/badge.svg)](https://github.com/michaeladler/shell-snoop/actions/workflows/build.yml)
[![ci](https://github.com/michaeladler/shell-snoop/actions/workflows/ci.yml/badge.svg)](https://github.com/michaeladler/shell-snoop/actions/workflows/ci.yml)

# shell-snoop

**shell-snoop** figures out the **exact command** which was used to run a child process in a shell.
This works great in combination with a session persistence plugin such as [tmux-resurrect](https://github.com/tmux-plugins/tmux-resurrect).

Currently, there is support for **bash** and **zsh**.

## Demo

```bash
# get pid of zsh
$ echo $$
14316

# start some arbitrary child process:
$ env foo=bar sleep 300
```

In another shell:

```bash
$ shell-snoop 14316
env foo=bar sleep 300
```

As you can see, `shell-snoop` was able to figure the exact command which was used to start the child process.

## Build & Install

```
$ cargo install
# setcap cap_sys_ptrace+eip ~/.cargo/bin/shell-snoop
```

**Hint**: A statically linked binary is provided as an artifact in [Github Actions](https://github.com/michaeladler/shell-snoop/actions/workflows/build.yml).
You must be logged in to download it though (this is a restriction imposed by Github for 'security reasons').

## Credits

This is based on [save_command_strategies/gdb.sh](https://github.com/tmux-plugins/tmux-resurrect/blob/8ebda79f6881d84a0cdc144ad5f20395eb0dd846/save_command_strategies/gdb.sh) by Bruno Sutic.
