#!/bin/bash
## #!/usr/bin/env bash
set -eux

uname -a
DESTDIR=/src/releases

mkdir -p $DESTDIR/i386-linux
rm -f $DESTDIR/i386-linux/*

mkdir -p $DESTDIR/arm-linux
rm -f $DESTDIR/arm-linux/*

function build {
	echo Building for $1 to $3...

	if [[ ! -f /build/$1/release/bliss-mixer ]]; then
		cargo build --release --target $1
	fi

	$2 /build/$1/release/bliss-mixer \
		&& cp /build/$1/release/bliss-mixer $DESTDIR/$3
}

build arm-unknown-linux-gnueabihf arm-linux-gnueabihf-strip arm-linux/bliss-mixer-hf
build aarch64-unknown-linux-gnu aarch64-linux-gnu-strip arm-linux/bliss-mixer-aarch64
build x86_64-unknown-linux-musl strip i386-linux/bliss-mixer-x86_64
build i686-unknown-linux-musl strip i386-linux/bliss-mixer
