# propdump2cell42
Create Active Worlds 4.2 cache files from a propdump

## Usage
1. Ensure blank42.idx and blank42.dat are in your current directory.
1. Pipe the propdump into the program, e.g. `propdump2cell42 < propdump.txt` or `zcat mbsurvey.txt.gz | propdump2cell42`
1. You're done

## Build notes

This was build on the stable-i686-pc-windows-msvc toolchain due to DLL requirements.
