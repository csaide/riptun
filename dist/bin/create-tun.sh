#!/bin/bash

# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: MIT

mkdir -p /dev/net
mknod /dev/net/tun c 10 200
chmod 0666 /dev/net/tun
