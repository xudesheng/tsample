# tsample

This purpose of this tool is to provide performance metrics for Thingworx.

From the version `v4.0.0`, this tool will not support to export the OS level metrics. Instead, it can support the export of the following metrics:

1. Thingworx Subsystems
2. Thingworx Connection Server
3. JMX C3P0 metrics (you need to install the extension first.)

## Install

The simplest way to install on the Ubuntu:

```
wget https://github.com/xudesheng/tsample/releases/download/v4.1.0/install.sh
chmod a+x install.sh
./install.sh
```

**Caution**: the above script will install InfluxDB, Grafana and Tsample together. You can manually install them one by one and configure them to your needs.


## How to use

### Help

``` shell
tsample -h
```

```
[2022-03-11T05:52:33Z INFO  tsample] tsample:4.0.0 Started.
[2022-03-11T05:52:33Z INFO  tsample] Log level: info, you can change it by setting TSAMPLE_LOG env.
tsample 4.0.0
xudesheng <xudesheng@gmail.com>


USAGE:
    tsample [OPTIONS]

OPTIONS:
    -c, --config <CONFIG_FILE>      Configuration file name, it should be a YAML file.
    -e, --export                    Export sample configuration file.
    -f, --flatten <FLATTEN_FILE>    Flatten the configuration file into a new yaml file for
                                    validation.
    -h, --help                      Print help information
    -V, --version                   Print version information
```



### Generate Sample Configuration File:

```
tsample -c myconfig.yml -e
```

With this command, it will create a yaml format configuration file.

### Adjust your configuration file:

The most important portion is:

```
thingworx_servers:
  - name: "platform1"
    # the hostname or IP address of the Thingworx Server, default is localhost
    host: "dxu-twx.demotest.io"
    # the port of the Thingworx Server, default is 8080
    port: 443
    # the protocol of the Thingworx Server, default is http. valid values are http and https
    protocol: https
    # the application name of the Thingworx Server, default is "Thingworx"
    # application: "Thingworx"

    # the appkey of the Thingworx Server, this is mandatory.
    app_key: "e5d38c56-c8da-4bff-bba3-06bf3da7474a"
```



### How to run

```
tsample -c myconfig.yml
```



### Workaround to delete all measurements from InfluxDB

```bash
DROP SERIES FROM /.*/
```

