PREFIX ?= /usr/local

CC ?= cc
CFLAGS ?= -std=c99 -Wall -Werror -Wno-error=unused-result -Os
LDFLAGS ?= -lcap-ng

build: out/shell-snoop-zsh

out/shell-snoop-zsh: src/shell-snoop.c
	[ -d out ] || mkdir -p out
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<
	ln -sf shell-snoop-zsh out/shell-snoop-bash

install:
	install -D -m0755 -s out/shell-snoop-zsh $(DESTDIR)$(PREFIX)/bin/shell-snoop-zsh
	cp -P -p out/shell-snoop-bash $(DESTDIR)$(PREFIX)/bin/

setcap:
	setcap cap_sys_ptrace+eip $(DESTDIR)$(PREFIX)/bin/shell-snoop-zsh

clean:
	rm -rf out
