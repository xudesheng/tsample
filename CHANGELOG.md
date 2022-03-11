# Changelog

All notable changes to this project will be documented in this file from the release of v4.0.0.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added

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
