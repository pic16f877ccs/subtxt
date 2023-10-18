## subtxt

Tool to hide text using image alpha channel.

## Description

Encodes text into a transparent area of the image, leaving the alpha channel transparent. Supports PNG and TIFF types.

### Build

Build with Rust package manager.

```console
cargo b -r
```

### Usage:

#### Input image with transparent area and input text.

```console
subtxt inputImage.png -i 'inputText.txt' -o 'outputImage.png'
```

#### Output invisible text.

```console
subtxt textInImage.png -O outputText.txt'
```

### Example:

#### Image from repository.
|<img title="Image with alpha channel" src="md_img/steck.png" alt="" width="325" height="">|
|:-:|

#### Hide text in image data.
```console
cargo run -r -- './md_img/steck.png' -i './md_img/subtxt.txt' -o './md_img/steck_subtxt.png'
```
|<img title="Image in image" src="md_img/steck.png" alt="" width="325" height="">|
|:-:|

#### Print invisible text.
```console
cargo run -r -- -p './md_img/steck_subtxt.png'

subtxt
Tool to hide text using image alpha channel.
Description
Encodes text into a transparent area of the image, leaving the alpha channel transparent. Supports PNG and TIFF types.
```

## License

GNU General Public License v3.0
