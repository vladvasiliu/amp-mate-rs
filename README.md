# Amp Mate

An interface for controlling audio amplifiers.

## Goals 

This projects aims to be a framework for interacting with "smart" audio equipment.

My main use case is controlling a Rotel amplifier from the computer.
The code aims to be general enough so as to be easily adapted to other audio equipment.

A CLI will be provided for interfacing with a taskbar (such as Polybar).

### Use cases

* Integrate with Volumio to provide volume control and source selection
* Control the amplifier with hotkeys and show its status in a taskbar of some sort.


## Supported environments


### Amplifier

This is tested on a Rotel RA-1572 with an ethernet connection.
As the IP protocol seems similar, other Rotel models shoud have at least basic support.

### OS

The CLI is fairly simple. The fanciest dependency is Tokio, so it should run pretty much everywhere.

Development happens mainly on Windows and Arch Linux, so these are the only tested environments.


## Useful links

* [Rotel RA-1572 RS232 / IP ASCII Controller Command List](http://rotel.com/sites/default/files/product/rs232/RA1572%20Protocol.pdf)