# Smart Device Mobile API

[![LICENSE][license badge]][license] [![Actions Status][actions badge]][actions] [![CodeCov][codecov badge]][codecov]

This repository is for a service application that allows the mobile application to initialize SIFIS-Home Smart Device. In addition, the repository is planned to have documentation for setting up demonstration Raspberry Pi.

## Project folders

* `debian` – Extra files for the Debian package
* `docs` – Documentation, instructions, plans
  * [Smart Device Mobile API](docs/Smart%20Device%20Mobile%20API.md) – Plans for Mobile API service and demo setup
* `scripts` – Example scripts for each server command
* `src` – Source files for the Mobile API library
  * `bin` – Source files for server and device information generator

* `static` – Static files served by the server
* `tests` – Integration tests
  * `scripts` – Testing scripts for checking that server runs them


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

For users of Debian-based distributions, an alternative guide after the generic Linux guide.

## Building for Raspberry Pi OS (64-bit) on any Linux

We have already added the following to the `.cargo/config.toml`

```toml
[build]

# Raspberry Pi 4 64-bit
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-none-linux-gnu-gcc"
strip = { path = "aarch64-none-linux-gnu-strip" }
```

This allows cross-compiling project for the 64-bit version of the Raspberry Pi OS. 

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

### Making package for Raspberry Pi OS

We use *cargo-deb* to create a Debian package for the Raspberry Pi OS. Install it with the following command, or skip it if you have it already.

```bash
$ cargo install cargo-deb
```

Give the following commands,  but change the first PATH to match your setup:

```bash
$ export PATH=$HOME/toolchains/gcc-arm-10.2-2020.11-x86_64-aarch64-none-linux-gnu/bin:$PATH
$ export TARGET_CC=aarch64-none-linux-gnu-gcc
$ export TARGET_AR=aarch64-none-linux-gnu-ar
$ cargo deb --target=aarch64-unknown-linux-gnu
```

See deploying instructions after Debian cross-compiling instructions




----



## Building for Raspberry Pi OS (64-bit) on Debian-based Distributions

### Install standard library

Assuming you already have a Rust development environment for the host system. Installing a standard library for our target system is easy as

```bash
$ rustup target add aarch64-unknown-linux-gnu
```

### Install cross building tools

We can install required tools with the command:

```bash
$ sudo apt install crossbuild-essential-arm64
```

We need to modify the `.cargo/config.toml` file to match tool names.  We need to remove `none` from the *linker* and *strip* names.

```toml
[build]

# Raspberry Pi 4 64-bit
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
strip = { path = "aarch64-linux-gnu-strip" }
```

### Making package for Raspberry Pi OS

We use *cargo-deb* to create a Debian package for the Raspberry Pi OS. Install it with the following command, or skip it if you have it already.

```bash
$ cargo install cargo-deb
```

Give the following commands,  but change the first PATH to match your setup:

```bash
$ export TARGET_CC=aarch64-linux-gnu-gcc
$ export TARGET_AR=aarch64-linux-gnu-ar
$ cargo deb --target=aarch64-unknown-linux-gnu
```




----



## Deploying to Raspberry Pi

Copy the Mobile API Server Debian package to the device, for example with scp:

```bash
$ scp target/aarch64-unknown-linux-gnu/debian/sifis-home-mobile-api_0.1.0_arm64.deb TARGET_DEVICE_ADDRESS:/home/developer
```

On the target device run the following command to install the package:

```bash
$ sudo dpkg -i sifis-home-mobile-api_0.1.0_arm64.deb
```

Create required device info with the command:

```bash
$ sudo create_device_info -s /home/developer/qr.svg "My Device Name"
```

The command above also creates QR code image, which allows the mobile application to scan it for API key needed to use server endpoints.

Finally start and enable the Mobile API Service:

```bash
$ sudo systemctl start mobile-api-server.service
$ sudo systemctl enable mobile-api-server.service
```



<!-- Links -->

[actions]: https://github.com/sifis-home/wp6_mobile_application_api/actions
[codecov]: https://codecov.io/gh/sifis-home/wp6_mobile_application_api
[license]: LICENSE

<!-- Badges -->

[actions badge]: https://github.com/sifis-home/wp6_mobile_application_api/workflows/mobile_api-ubuntu/badge.svg
[codecov badge]: https://codecov.io/gh/sifis-home/wp6_mobile_application_api/branch/master/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
