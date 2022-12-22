# Smart Device Mobile API

[![LICENSE][license badge]][license] [![Actions Status][actions badge]][actions] [![CodeCov][codecov badge]][codecov]

This repository is for a service application that allows the mobile application to initialize SIFIS-Home Smart Device. In addition, the repository is planned to have documentation for setting up demonstration Raspberry Pi.

## Project folders

* `create_device_info` – Tool for creating device.json file for the server and Qr Code for the Mobile application.
* `docs` – Documentation, instructions, plans
  * [Smart Device Mobile API](docs/Smart%20Device%20Mobile%20API.md) – Plans for Mobile API service and demo setup
* `mobile_api` – Shared library for Smart Device applications
* `mobile_api_server` – Smart Device Mobile API server program
  * `static` – Static files served by the server
  * `scripts` – Example scripts for each server command

## Building

**Note:** Server works only in Linux, and probably with the macOS.

Install [Rust](https://www.rust-lang.org/tools/install) development environment.

```bash
$ cd project_folder
$ cargo build
```

## Run server

Server requires `device.json` file to start. We can use `create_device_info` tool to do that. The tool can also create Qr code for the Mobile Application to know authorization key.

```bash
$ cargo run --bin=create_device_info -- --save-qr-code-svg code.svg "Product name"
```

We can now start the server:

```bash
$ cargo run
```

The API documentation is available from the server by opening the URL http://127.0.0.1:8000 with the web browser.

# Cross Compiling

## Raspberry Pi OS (64-bit)

### Already done

We have already added the following to the `.cargo/config.toml`

```toml
[build]

# Raspberry Pi 4 64-bit
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-none-linux-gnu-gcc"
```

This allows cross-compiling for the 64-bit version of the Raspberry Pi OS. 

### Install standard library

Assuming you already have a Rust development environment for the host system. Installing a standard library for our target system is easy as

```bash
$ rustup target add aarch64-unknown-linux-gnu
```

### Download GNU Toolchain

We need to install the correct version of the toolchain, which we can check by running the following command on Raspberry Pi:

```bash
$ gcc --version
gcc (Debian 10.2.1-6) 10.2.1 20210110
Copyright (C) 2020 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
```

Go to [https://developer.arm.com/downloads/-/gnu-a](https://developer.arm.com/downloads/-/gnu-a) and download the toolchain that matches your host system and target system.

For us, the host system was **x86_64 Linux** and target system needed **10.2** version of the toolchain. Therefore we downloaded:
 `gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu.tar.xz`

Download and uncompress the toolchain at the location of your choice.

### Compiling to Raspberry Pi

Give the following commands,  but change the first PATH to match your setup:

```bash
$ export PATH=$HOME/toolchains/gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu/bin:$PATH
$ export TARGET_CC=aarch64-none-linux-gnu-gcc
$ export TARGET_AR=aarch64-none-linux-gnu-ar
$ cargo build --release --target=aarch64-unknown-linux-gnu
```

### Deploying to Raspberry Pi

You can modify the following script to match your system.

```bash
#!/bin/bash

# Download toolchain from: https://developer.arm.com/downloads/-/gnu-a
# Change path if necessary
export PATH=$HOME/toolchain/gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu/bin:$PATH

# Using SSH to transfer binary to Pi
readonly TARGET_HOST=sd-sifis-home
readonly TARGET_PATH=/home/developer/bin
readonly TARGET_BINARY=mobile_api_server

# No need to edit these
readonly TARGET_ARCH=aarch64-unknown-linux-gnu
export TARGET_CC=aarch64-none-linux-gnu-gcc
export TARGET_AR=aarch64-none-linux-gnu-ar

# Building as release version for Raspberry Pi 4
cargo build --release --target=${TARGET_ARCH}

# Copy server binary to Pi
scp target/${TARGET_ARCH}/release/${TARGET_BINARY} ${TARGET_HOST}:${TARGET_PATH}

# Run server on Pi
ssh -t ${TARGET_HOST} ROCKET_ADDRESS=0.0.0.0 ROCKET_LOG_LEVEL=normal ${TARGET_PATH}/${TARGET_BINARY}
```



Please note that the server needs a `device.json` file, `static` folder, and `scripts` folder. The server expects to find these files from `/opt/sifis-home directory`. 

Create `device.json` with the `create_device_info` on the host system and copy it and other files manually to the correct locations.

The target system should have something like this:

<pre style="line-height:1em;">
<b>/opt/sifis-home</b>
├── device.json
├── <b>scripts</b>
│   ├── factory_reset.sh
│   ├── restart.sh
│   └── shutdown.sh
└── <b>static</b>
    ├── favicon.ico
    └── index.html
</pre>

<!-- Links -->

[actions]: https://github.com/sifis-home/wp6_mobile_application_api/actions
[codecov]: https://codecov.io/gh/sifis-home/wp6_mobile_application_api
[license]: LICENSE

<!-- Badges -->

[actions badge]: https://github.com/sifis-home/wp6_mobile_application_api/workflows/mobile_api-ubuntu/badge.svg
[codecov badge]: https://codecov.io/gh/sifis-home/wp6_mobile_application_api/branch/master/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
