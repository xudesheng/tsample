# tsample

This purpose of this tool is to provide performance metrics for Thingworx.

From the version `v4.0.0`, this tool will not support to export the OS level metrics. Instead, it can support the export of the following metrics:

1. Thingworx Subsystems
2. Thingworx Connection Server
3. JMX C3P0 metrics (you need to install the specific extension first.)
4. Persistent Property metrics (you need to install the specifc extension first.)

* reaching out to PTC TS or your service provider to get the extensions.

## Install

### Provided builds
- [x] Debian format (`deb`) package, best for Debian based systems, like: Ubuntu.
- [ ] RPM format (`rpm`) package, best for RedHat based systems, like: Redhat. Please submit an issue to request this build, and tell me which distribution you are using, like: CentOS, Redhat, Fedora etc. The RPM build will be slightly different between the distributions.
- [x] Windows (`exe`) package, best for Windows based systems, x86_64 only.
- [x] MacOS executable binary, x86_64.
- [x] MacOS Apple Silicon (M1) binary, aarch64. This build will be added manually. In case I forgot to add this build, please submit an issue to request this build.
- [x] Linux aarch64 binary. This build will be added manually. In case I forgot to add this build, please submit an issue to request this build.

** All Linux build will be statically linked. **

** Windows build will be statically linked. **

### Installation

#### Debian (Ubuntu) based Linux installation

1. Download the `deb` format from the latest release
2. ```sudo dpkg -i <the downloaded deb file>```
3. Configure `/etc/tsample/tsample.yaml` based on your needs.
4. ```sudo systemctl enable tsample``` (this only needs one time)
5. ```sudo systemctl start tsample```

#### Manually Install and configuration

1. The `install.sh` can be a good reference if you are installing on Linux.
2. Download the build based on your OS
3. Export a sample configuration file: ```./tsample -c myconfig.yaml -e```
4. Modify your configuration file: `myconfig.yaml`
5. Start to execute it: ```./tsample -c myconfig.yaml```




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

