# 🔥 vcontrol-rs

[![Build Status](https://travis-ci.com/reitermarkus/vcontrol-rs.svg?branch=master)](https://travis-ci.com/reitermarkus/vcontrol-rs)
[![Crates.io](https://img.shields.io/crates/v/vcontrol.svg)](https://crates.io/crates/vcontrol)
[![Documentation](https://docs.rs/vcontrol/badge.svg)](https://docs.rs/vcontrol)

This is a Rust library for communication with Viessmann heating controllers.

The included `Optolink` struct is a low-level abstraction for an Optolink connection over either a TCP socket or a serial port.

The `VControl` struct is a high-level abstraction for a complete system, which can be configured with a YAML file, much like what [`vcontrold`](https://github.com/openv/vcontrold) does with an XML file.
