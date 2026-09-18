#!/usr/bin/bash

./target/release/k99-landfill \
--exec /usr/bin/ls \
--exec /lib/ld-linux-x86-64.so.2 \
--ro /lib/libcap.so.2 \
--ro /lib/libc.so.6 \
--ro "$HOME" \
-- ls "$HOME"
