#!/bin/bash
tempfile=`mktemp`
cargo espflash save-image --chip esp32 --merge ${tempfile}
qemu-system-xtensa -nographic -machine esp32 -drive file=${tempfile},if=mtd,format=raw
rm ${tempfile}