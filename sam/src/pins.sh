#!/bin/bash

# P8 GPIO (modified for rev4 ground)
config-pin p8.7 gpio
config-pin p8.8 gpio
config-pin p8.9 gpio
config-pin p8.10 gpio
config-pin p8.11 gpio
config-pin p8.12 gpio
config-pin p8.13 gpio
config-pin p8.14 gpio
config-pin p8.15 gpio
config-pin p8.16 gpio
config-pin p8.19 gpio
config-pin p8.21 gpio
config-pin p8.23 gpio
config-pin p8.25 gpio

# P9 GPIO (modified for rev4 ground)
config-pin p9.11 gpio
config-pin p9.12 gpio
config-pin p9.13 gpio
config-pin p9.14 gpio
config-pin p9.15 gpio
config-pin p9.16 gpio
config-pin p9.23 gpio
config-pin p9.24 gpio
config-pin p9.25 gpio
config-pin p9.26 gpio
config-pin p9.27 gpio
# The following will likely need a kernel modification to work
config-pin p9.28 gpio

# SPI 0 (slow) modified for rev4 ground
config-pin p9_18 spi
config-pin p9_21 spi
config-pin p9_22 spi_sclk

# SPI 1 (fast) modified for rev4 ground
config-pin p9_19 spi_cs
# config-pin p9_28 spi_cs (I was stupid and made this a drdy pin)
config-pin p9_29 spi
config-pin p9_30 spi
config-pin p9_31 spi_sclk