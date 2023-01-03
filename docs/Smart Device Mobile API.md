# Smart Device Mobile API

The Smart Device Mobile API allows the Mobile application to initialize new devices to the SIFIS-Home network. In addition, the following functionality has been planned:

* Reporting device status
    * CPU usage
    * Memory usage
    * Disk space usage
    * Uptime
    * Load average
* Sending commands
    * Factory reset settings
    * Restart device
    * Shut down device

## Endpoints

* Device information and configuration

  * [GET] device/status

  * [GET,PUT] device/configuration

* Commands

  * [GET] command/factory_reset
  * [GET] command/restart
  * [GET] command/shutdown

## Smart Device Initialization

![Smart Device Initialization Sequence](images/mobile-api-init.svg)

The Smart Device Mobile API will run on Smart Device, and the Mobile Application connects to it using Wi-Fi. An authorization key is needed to use the API, and it is planned to be delivered with Smart Device in a QR code form. The mobile application can scan a QR code with the camera to receive the authorization key.

The mobile application retrieves device information using the authorization key. The information contains the device name and unique identifier. Then, the mobile application presents the information to the user and helps the user choose the correct device if several devices are available.

The mobile application defines the device's configuration, represented in the following subsection. After the configuration has been sent, the mobile application asks the device to restart. After the restart, the new Smart Device becomes part of the SIFIS-home network.

### Device Configuration

The configuration file is stored on the device in the `/opt/sifis-home/config.json` file. The presence of this file allows Smart Device to join the SIFIS-Home network. Without the file, the device starts in the initialization mode, where it creates Wi-Fi access point for the mobile application.

All SIFIS-Home services running on the device can read this file, but only the Smart Device Mobile API is allowed to write it. Other SIFIS-Home services are not started on the device if the file is missing.

The configuration file contains:

* Device name (user defined)
* Shared key for DHT

Configuration file mockup:

```json
{
    "dht-shared-key": "32 bytes in hex format",
    "name": "User-defined name for the device"
}
```


### Network Configuration

The Smart Device Mobile API also stores network configuration defined by the user in the system's network configuration files. These settings can be cleared with the factory reset function.

* Connection type
     * Wired
     * Wireless
         * Access point SSID
         * Security (WEP/WPA/WPA2)
* Address
     * From DHCP
     * Static

In the mobile application, static network settings could be behind the advanced settings button, and the device uses DHCP by default.

## Device Information File

The idea behind this file is that it is written at the factory. The file contains the unique data of the device and the authorization key. We create this file ourselves for demonstrations or let the Smart Device Mobile API make it.

The file is stored on the device in the `/opt/sifis-home/device.json` and contains the following information.

* Product name
* Unique identifier
* Authorization key
* Private key location (*sifis-dht generates on the first run*)

Some or all of these are delivered with the device in a QR code for the mobile application to scan.

Device info file mockup:

```json
{
    "authorization-key": "256-bits in hex format",
    "private-key-file": "/opt/sifis-home/private.pem",
    "product-name": "Name of the product (not unique)",
    "uuid": "128-bit UUID in standard hex format"
}
```

## Smart Device Boot

```mermaid
%%{ init: { 'flowchart': { 'curve': 'basis' } } }%%
flowchart
A[Boot] -->B{config.json?}
    
    B -->|present| C(Setup network interfaces)
    C --> D[Start SIFIS-Home services]
    
    B -->|missing| E(Start configuration AP)
    E --> F[Smart Device Mobile API]
    F -->|Writing config.json| F
    F -->|restart| A
```

### SIFIS-Home Targets

The systemd is probably the most common solution for managing services on Linux systems. Below is a simplified graph of default boot targets and services to the multi-user target. The multi-user target has everything running except the graphical user interface and was chosen for the example as SIFIS-Home devices likely do not have displays attached to them.

![Dependencies for default multi-user target boot](images/default-boot.svg "Dependencies for default multi-user target boot")

We can create two new targets for the SIFIS-Home device, the selection of which target is active is based on whether the `config.json` file exists. Now we can set up systemd services to be wanted by one of the targets to decide whether they are run. Services that are only needed for configuration are installed under `sifis-config.target`, and services for the fully configured system are installed under `sifis-home.target`.  The image below shows added targets with their conditions.

![Adding SIFIS-Home Targets](images/sifis-home-boot.svg "Adding SIFIS-Home Targets")

**Note**: Other targets relevant to boot were left out of the picture to clarify SIFIS-Home target additions. All targets and services left out of the picture still exist. We only add two new targets and do not remove anything.

These targets are added to the `/etc/systemd/system` directory.

___

`sifis-config.target` example:

```ini
# The target for an unconfigured SIFIS-Home system

[Unit]
Description=SIFIS-Home Configuration Mode
Wants=network.target
After=network.target
ConditionPathExists=!/opt/sifis-home/config.json
Conflicts=rescue.service rescue.target shutdown.target

[Install]
WantedBy=multi-user.target
```

___

`sifis-home.target` example:

```ini
# The target for a fully configured SIFIS-Home system

[Unit]
Description=SIFIS-Home System
Wants=network.target
After=network.target
ConditionPathExists=/opt/sifis-home/config.json
Conflicts=rescue.service rescue.target shutdown.target

[Install]
WantedBy=multi-user.target
```

___

We can enable these targets with the following commands:

```bash
$ sudo systemctl enable sifis-config.target
$ sudo systemctl enable sifis-home.target
```

