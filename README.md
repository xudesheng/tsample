# tsample

This purpose of this tool is to provide performance metrics for Thingworx.

It can export different kind of performance metrics not only from Thingworx but also from OS level. However, it seems like `Telegraf` already provides enough OS level metrics, therefore, OS level export will be limited and be optional.

### How to deploy Telegraf on Windows

How to install it as Windows service: https://github.com/influxdata/telegraf/blob/master/docs/WINDOWS_SERVICE.md

Download: https://dl.influxdata.com/telegraf/releases/telegraf-1.13.0_windows_amd64.zip



### How to deloy Telegraf on Linux

How to install on Redhat/CentOS

```bash
wget https://dl.influxdata.com/telegraf/releases/telegraf-1.13.0-1.x86_64.rpm
sudo yum localinstall telegraf-1.13.0-1.x86_64.rpm
```

How to install on Ubuntu:

```bash
wget https://dl.influxdata.com/telegraf/releases/telegraf_1.13.0-1_amd64.deb
sudo dpkg -i telegraf_1.13.0-1_amd64.deb
```

How to configure:

https://github.com/influxdata/telegraf/blob/master/docs/CONFIGURATION.md



### Workaround to delete all measurements from InfluxDB

```bash
DROP SERIES FROM /.*/
```

