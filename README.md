# wavelog-sync

## Overview
`wavelog-sync` connects to one or more radios through `rigctld` and forwards live frequency and mode data to a [Wavelog](https://github.com/wavelog/wavelog) instance.  
It is designed for unattended operation on Linux systems and can be run manually or as a system service.

---

## Principle of Operation
1. Each transceiver is controlled by a local or remote `rigctld` daemon (part of `hamlib`).
2. `wavelog-sync` periodically queries every configured rig for its current frequency and mode.
3. The collected information is sent to a Wavelog API endpoint using your user token.

This enables Wavelog to display your active band and mode in real time.

---

## Configuration

The configuration file (`config.yaml`) defines your rigs and Wavelog connection:

```yaml
rigs:
  - name: FTDX10
    address: 127.0.0.1:12345
  - name: IC9700
    address: 127.0.0.1:12346
wavelog:
  address: https://log.mydomain.com/index.php/api/radio
  token: YOUR-WAVELOG-TOKEN
```

You have to create a token in your wavelog instance.

## Running as a service (systemd)

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
