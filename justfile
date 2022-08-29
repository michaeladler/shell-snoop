
musl-build *args:
    cargo build --target x86_64-unknown-linux-musl {{args}}

update-deps:
    #!/usr/bin/env bash
    set -eux
    nix flake update
    cargo update
    git commit -am "chore: update deps"
    nix build
    git push
