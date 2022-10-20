# heatsrhink

This is a modified version of the decoder from https://github.com/snakehand/heatshrink


**Description**

The main difference is you decompress/read the decompressed data byte per byte.

It is useful for use cases such as font compression etc.. where the output can be big
and you process them in little chunks

_/!\ The window size is hardcoded to be 7 maximum, look ahead 5 maximum /!\\_

This is to cap the consumed buffer memory to about 256 bytes

**Original description :**

Minimal no_std implementation of Heatshrink compression &amp; decompression
