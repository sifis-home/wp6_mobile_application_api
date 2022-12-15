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

<!-- Links -->

[actions]: https://github.com/sifis-home/wp6_mobile_application_api/actions
[codecov]: https://codecov.io/gh/sifis-home/wp6_mobile_application_api
[license]: LICENSE

<!-- Badges -->

[actions badge]: https://github.com/sifis-home/wp6_mobile_application_api/workflows/mobile_api-ubuntu/badge.svg
[codecov badge]: https://codecov.io/gh/sifis-home/wp6_mobile_application_api/branch/master/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
