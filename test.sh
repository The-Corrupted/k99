#!/usr/bin/bash

./target/release/k99-landfill \
--exec /usr/bin/true \
--exec /lib/ld-linux-x86-64.so.2 \
--ro /usr/lib/libc.so.6 \
-- /usr/bin/true
