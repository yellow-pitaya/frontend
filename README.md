# Yellow Pitaya

This is another flover of pitaya, same hardware but different software.

![](screenshot.png)

This is a desktop interface for [redpitaya](https://redpitaya.com/).

## Install

```
cargo build
```

## Configuration

Enable SCPI server on redpitaya : http://rp-xxxxxx.local/scpi_manager/

Or via ssh:

```
systemctl start redpitaya_scpi.service
```

You can permantly enable it on startup:

```
systemctl enable redpitaya_scpi.service
```

This will disable web applications.
