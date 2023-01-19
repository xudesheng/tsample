# Changelog

All notable changes to this project will be documented in this file from the release of v4.0.0.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Changed
## [v4.3.4] - 2023-01-19
 - Update build cache in order to solve the issue: https://github.com/rustls/rustls/issues/1012

### Added
## [v4.3.3] - 2022-07-05

### Changed
 - Update cargo lock in order to remove the dependency on `smallvec` old version
 - Change rust builder version to 1.62.0

## [v4.3.1] - 2022-05-011

### Changed
 - Support to expose metrics in Prometheus format (http://localhost:19090/metrics).
 - Changed the response time unit from nano to milliseconds.

## [v4.3.0] - 2022-05-011

### Added
 - Support to expose metrics in Prometheus format (http://localhost:19090/metrics).

## [v4.2.4] - 2022-05-05

### Changed
 - Fixed the bug: when a query is timed out, the sleep time was not being reset due to u64 overflow.

## [v4.2.3] - 2022-04-07

### Added
 - Limited support to flatten the Java TabularData format into InfluxDB Measurement for GC metrics.
 - Added sample of `jmx_garbagecollector_status` to capture the GC metrics in the sample configuration file.

## [v4.2.2] - 2022-04-07

### Changed
 - Support "double" or "java.lang.Double" for the "f64" type. it will be a float in InfluxDB.

## [v4.2.1] - 2022-03-25

### Changed
 - Removed company Email address from the sample configuration file.

## [v4.2.0] - 2022-03-25

### Added
 - Added `jmx_metrics` in the config, you can grab all possible jmx metrics now.
 - added `jmx_c3p0_connections` and `jmx_memory_status` as two examples in the config.

### Deleted
 - deleted `c3p0_metrics` in the config, it is not used anymore.

## [v4.1.2] - 2022-03-11

### Added
 - Added `deb` format for downloading. Debian based user can directly install this package.

### Changed
 - x86_64 builds will be generated automatically via github actions.
 - aarch64 (including Apple Silicon) builds will be generated manually.

## [v4.1.1] - 2022-03-11

### Changed
 - start to use new release workflow

## [v4.1.0] - 2022-03-11

### Added
 - support to grab metrics from arbitrary URLs
 
## [v4.0.1] - 2022-03-11

### Changed
 - This release is only a test for github action workflow.
 
## [v4.0.0] - 2022-03-09

### Changed
 - Replaced the `influxdb` client driver with the one from the vendor. The original driver was built by myself 3+ years ago and I don't want to maintain it anymore.
 - Upgraded the Rust version to the latest stable version (2021 edition 1.59).
 - Upgraded tokio library from 0.2 to 1.17. A big jump.
 - Replaced the error lib from `failure` to `anyhow`
 - Changed the configuration file format from TOML to YAML for readability.
 - Changed the optional CSV output format to CSV but JSON payload.
 - Support to dynamically grasp the available connection servers.
 - Support to summarize the result from the JMX extension output,mainly focus on the C3P0 driver metrics.
 - Replace `native-tls` with `rustls` for the TLS library. This will remove the depencency of the OpenSSL installation.
 - Added github action to build and release the multiple targets executable files, instead of using local dockers.

