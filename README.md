# wavelog-sync

## Overview
`wavelog-sync` connects to one or more radios through [rigctld](https://hamlib.sourceforge.net/html/rigctld.1.html) and forwards live frequency and mode data to a [Wavelog](https://github.com/wavelog/wavelog) instance.

---

## Principle of Operation
1. Each transceiver is controlled by a local or remote `rigctld` daemon (part of `hamlib`).
2. `wavelog-sync` periodically queries every configured rig for its current frequency and mode.
3. The collected information is sent to a Wavelog API endpoint using your user token.

This enables Wavelog to display your active band and mode in real time.

---

## Installation

prerequisites:
* Linux with systemd (or windows/macos but no instructions provided for these)
* Rust toolchain with Cargo
* hamlib with rigctld
* A Wavelog API token and API URL from your instance

build from source:
```
git clone https://github.com/HB9HUS/wavelog-sync.git
cd wavelog-sync

cargo build --release

sudo install -m 0755 target/release/wavelog-sync /usr/local/bin/wavelog-sync
```

create config file:
```
sudo mkdir -p /etc/wavelog-sync
sudo cp config.yaml /etc/wavelog-sync/config.yaml
sudo chmod 640 /etc/wavelog-sync/config.yaml
```

## Configuration

The configuration file (`config.yaml` or `-c myconfig.yaml`) defines your rigs and Wavelog connection:

```yaml
rigs:
  - name: FTDX10
    address: 127.0.0.1:12345
    power_scale: 100
    send_power: true
  - name: IC9700
    address: 127.0.0.1:12346
    power_scale: 100
    send_power: false
wavelog:
  address: https://log.mydomain.com/index.php/api/radio
  token: YOUR-WAVELOG-TOKEN
```

You have to create a token in your wavelog instance.

If you need more logging, call with RUST_LOG=LEVEL (error|warn|info|debug)
    RUST_LOG=debug wavelog-sync -c myconfig.yaml

## Running as a service (systemd)

create a user (add to dialout group to allow access to devices):
```
  sudo useradd -r -G dialout -s /usr/sbin/nologin rigctl
```

Create a service file: /etc/systemd/system/wavelog-sync.service

```
  [Unit]
  Description=Rig2Wavelog service
  After=network.target

  [Service]
  ExecStart=/usr/local/bin/wavelog-sync
  WorkingDirectory=/etc/wavelog-sync
  Restart=always
  User=rigctl
  Group=rigctl

  [Install]
  WantedBy=multi-user.target
```

Then enable and run:
```
  sudo systemctl daemon-reload
  sudo systemctl enable wavelog-sync
  sudo systemctl start wavelog-sync
```

## Running rigctld as a service

For each rig create a file such as /etc/systemd/system/rigctld-ic9700.service

```
  [Unit]
  Description=rigctld for IC9700
  After=network.target

  [Service]
  ExecStart=/usr/bin/rigctld -m 3073 -r /dev/ttyUSB0 -s 115200 -t 12346
  Restart=always
  User=rigctl
  Group=rigctl

  [Install]
  WantedBy=multi-user.target
```

Then enable and run:

```
  sudo systemctl daemon-reload
  sudo systemctl enable rigctld-ic9700
  sudo systemctl start rigctld-ic9700
```

## Name your devices [Optional]

If you want to make sure, that each rig is running on a defined device, you can use udev.

Use this command to list the properties of your device:
```
  udevadm info -n /dev/ttyUSB2 --query=property
```

create a file /etc/udev/rules.d/99-radios.rules. These are just examples how they could look. Create for your own devices:
```
  SUBSYSTEM=="tty", KERNEL=="ttyUSB*", ENV{ID_SERIAL_SHORT}=="00F8CFDB", ENV{ID_USB_INTERFACE_NUM}=="00", SYMLINK+="ftdx10"
  SUBSYSTEM=="tty", KERNEL=="ttyUSB*", ENV{ID_SERIAL_SHORT}=="IC-9700_13008980_A", ENV{ID_USB_INTERFACE_NUM}=="00", SYMLINK+="icom9700"
```

and then:
```
  sudo udevadm control --reload-rules
  sudo udevadm trigger
```

Then you have specific devices for your radios such as /dev/ftdx10 that you can use in the rigctld config.

## Automatically Start/Stop services [Optional]

I recommend to disable rigctld when no rig is present in the system. You can detect this over one of the devices the rig provides. As tty devices sometimes still exist when the device is turned off (observed with Yaesu rigs), it is better to take a high level device such as the Audio Device.

Find it with lsusb:
```
  lsusb

  Bus 003 Device 028: ID 0d8c:0013 C-Media Electronics, Inc. USB Audio Device
```

and create a udev rule that creates a device and starts the service.
```
  SUBSYSTEM=="usb", ATTRS{idVendor}=="0d8c", ATTRS{idProduct}=="0013", TAG+="systemd", SYMLINK+="rig_audio_present", ENV{SYSTEMD_WANTS}="rigctld-ic710.service"

```

Update the service description to use the new device:
```
  [Unit]
  Description=rigctld for FT-710
  After=network.target
  BindsTo=dev-rig_audio_present.device
  After=dev-rig_audio_present.device

  ...
```

### Background info
rigctld can not realiably detect if a rig is connected or powered. In some situation in generates unwanted output which can be sent to wavelog. wavelog-sync tries to detect this but it is recommended to only turn on rigctld when the rig is connected.

Some rigs have their USB controller powered over USB and not the rig itself. This leads to the situation, that the devices ttyUSB0 etc. are still present in the system even when turned off. To detect this, you can try to see a higher-level device such as the Audio device. As the audio codec seems to get it's power from the rig and not the USB controller, this device disappears when the rig is turned off.
