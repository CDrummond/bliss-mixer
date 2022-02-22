#!/bin/bash
## #!/usr/bin/env bash
set -eux

uname -a
DESTDIR=/src/releases

for d in armhf-linux arm-linux x86_64-linux ; do
    mkdir -p $DESTDIR/$d
    rm -f $DESTDIR/$d/*
done

function build {
	echo Building for $1 to $3...

	if [[ ! -f /build/$1/release/bliss-mixer ]]; then
		cargo build --release --target $1
	fi

	$2 /build/$1/release/bliss-mixer && cp /build/$1/release/bliss-mixer $DESTDIR/$3
}

build arm-unknown-linux-gnueabihf arm-linux-gnueabihf-strip armhf-linux/bliss-mixer
build aarch64-unknown-linux-gnu aarch64-linux-gnu-strip arm-linux/bliss-mixer
build x86_64-unknown-linux-musl strip x86_64-linux/bliss-mixer

