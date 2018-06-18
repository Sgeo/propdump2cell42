# propdump2cell42
Create Active Worlds 4.2 cache files from a propdump

## Usage
1. Download and unzip from https://github.com/Sgeo/propdump2cell42/releases
1. Ensure blank42.idx and blank42.dat are in your current directory.
1. Pipe the propdump into the program, e.g. `propdump2cell42 < propdump.txt` or `"C:\Program Files\7-Zip\7z.exe" x mbsurvey.txt.gz -so | propdump2cell42`
1. You're done

## Alphaworld notes
The Alphaworld propdump, available on https://archive.org/details/alphaworld_propdump_2017_10_11 , is roughly 20 GB uncompressed. A number of design decisions were taken to allow usage of this program without needing to store a 20 GB file on disk.

If you have 7-Zip installed, it can be used to decompress and stream the mbsurvey.txt.gz file, as shown: `"C:\Program Files\7-Zip\7z.exe" x mbsurvey.txt.gz -so | propdump2cell42`

Due to the size of the propdump, only segments of Alphaworld can be viewed at one time. See below for how to view wanted areas

## Selection

Active Worlds 4.2 can only process cache files that are 2 GB or less in size. This program allows options to select interesting areas:
* `-c` or `--citnum`: A list of citnums. The resulting files will only have property by those citnums. E.g.: `"C:\Program Files\7-Zip\7z.exe" x mbsurvey.txt.gz -so | propdump2cell42 -c 1 99` will result in cache files that only contain property owned by citizens 1 and 99.
* `-t` or `--teleports` and `-r` or `--radius`: If t/teleports is used, r/radius must also be used. These options allow selecting a list of locations and how much area around them to include.

# Example

teleport.txt contains the following:
```
AW 0N 0W
AW 2222S 2222E
```

The command `"C:\Program Files\7-Zip\7z.exe" x mbsurvey.txt.gz -so | propdump2cell42 -t teleport.txt -r 100` will result in cache files that contain 100S 100E thru 100N 100W and 2322S 2322E thru 2122S 2122E.


## Build notes

This was build on the stable-i686-pc-windows-msvc toolchain due to DLL requirements.
