# Palantir

Palantir is a Work In Progress P2P networking library written in Rust. 

At the time of viewing, there may not be any code published yet. I commonly keep Work-In-Progress libraries private until they are ready for release, however for this one I want to get to an MVP as fast as possible. While this is currently published with no code at version 0.0.0, there will be many sudden very breaking changes. For this purpose, until version 1.0.0 is released, every minor version can be considered to contain breaking changes. This is the same for other projects including [Fluxion](https://github.com/peperworx/fluxion), and a library to integrate this one with Fluxion (that is still in progress).

## Palantir's Expected Design

Peer to Peer networking is complicated, but Peer to Peer libraries don't have to be. While existing libraries are very robust and extremely extensible, they tend to have rather complicated interfaces and sometimes feel more like frameworks than libraries. Palantir will be designed to be very simple.

Palantir will be based off of "layers". Each layer will provide some functionality, and will wrap additional layer(s). The layer lowest in the tree, that does not wrap any other layers, is known as a "transport".

Each layer allows sending bytes to a peer.

## Why Is This Being Made?

I am making Palantir primarily for use with Fluxion. Fluxion enables messages to be sent between different actor systems. Palantir will enable sending messages to different computers on different networks without (much) backend infrastructure. I hope that this will pair well with Fluxion for building peer-to-peer applications.


It should be noted that there is no exact timeframe for this project to reach a usable state, however planning and work is beginning imminently.