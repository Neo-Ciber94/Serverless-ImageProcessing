# Serverless Image Processing

[![CI](https://github.com/Neo-Ciber94/Serverless-ImageProcessing/actions/workflows/ci.yml/badge.svg)](https://github.com/Neo-Ciber94/Serverless-ImageProcessing/actions/workflows/ci.yml)

A Rust serverless application to manipulate images deployed using AWS with CDK.

## To run locally using `axum`

```bash
cargo run --features local --bin <name of the binary>
```

The binaries are the 2 serverless functions: `get_image` and `post_image` for example:

`cargo run --features local --bin get_image`

## Features

- [x] Read images from external sources
- [x] Read images from `form-data`
- [x] Read images from `base64` data.
- [ ] Image operations:
  - [x] Resize
  - [x] Crop
  - [x] Quality
  - [x] Blur
  - [x] Brightness
  - [x] Contracts
  - [x] Hue
  - [x] Flip (horizontal, vertical)
  - [x] Grayscale
  - [x] Invert
  - [x] Sharp (reverse blur)
  - [ ] Change output format
- [ ] Add swagger or postman integration

## Endpoints

- `GET /`

  - Query parameters
    - `source_url`: URL of the image to get.
    - `source_base64`: The base64 encoded image.
    - `width`: The width to resize the image to.
    - `quality`: The quality to apply to the resulting image. (0 - 100)
    - `brightness`: The brightness to apply to the resulting image.
    - `contrast`: The contrast to apply to the resulting image.
    - `hue`: The hue to recolor de image.
    - `sharp`: The amount of sharpness to unblur the image.
    - `flip`: "vertical" or "horizontal" value to rotate the image.
    - `grayscale`: "true" or "false" value to grayscale the image.
    - `crop`: The points to crop the image, this require the following separate query parameters to work:
      - `crop_x`
      - `crop_y`
      - `crop_width`
      - `crop_height`

- `POST /`
  - Body
    - `form-data` containing the image to process.
    - JSON body in the form: `{ "base64_data": "<base64 encoded image>" }`
  - Query parameters
    - `source_url`: URL of the image to get.
    - `source_base64`: The base64 encoded image.
    - `width`: The width to resize the image to.
    - `quality`: The quality to apply to the resulting image. (0 - 100)
    - `brightness`: The brightness to apply to the resulting image.
    - `contrast`: The contrast to apply to the resulting image.
    - `hue`: The hue to recolor de image.
    - `sharp`: The amount of sharpness to unblur the image.
    - `flip`: "vertical" or "horizontal" value to rotate the image.
    - `grayscale`: "true" or "false" value to grayscale the image.
    - `crop`: The points to crop the image, this require the following separate query parameters to work:
      - `crop_x`
      - `crop_y`
      - `crop_width`
      - `crop_height`
