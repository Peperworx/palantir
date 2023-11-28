# Palantir

Palantir is a Work In Progress P2P networking library written in Rust. 

At the time of viewing, there may not be any code published yet. I commonly keep Work-In-Progress libraries private until they are ready for release, however for this one I want to get to an MVP as fast as possible. While this is currently published with no code at version 0.0.0, there will be many sudden very breaking changes. For this purpose, until version 1.0.0 is released, every minor version can be considered to contain breaking changes. This is the same for other projects including [Fluxion](https://github.com/peperworx/fluxion), and a library to integrate this one with Fluxion (that is still in progress).

## Palantir's Expected Design

Peer to Peer networking is complicated. Palantir will start out very simple: a hosting server that relays messages between clients and itself. While this is a very simple design that is not-quite-peer-to-peer, this will help to lay most of the framework for Palantir's API, as the actual method used for sending messages will be abstracted over with traits. Palantir will try to remain as future proof as possible. As such, the primary protocol used for the initial server implementation will be WebTransport. WASM support is expected in the future, however it may take some time. No-Std support is not a primary goal, due to the nature of cryptography and IO implementations.

## Why Is This Being Made?

I am making Palantir primarily for use with Fluxion. Fluxion enables messages to be sent between different actor systems. Palantir will enable sending messages to different computers on different networks without (much) backend infrastructure. Combining the two will yield decentralized, distributed, and scalable applications. This combination will be used in future projects with much more in store.

It should be noted that there is no exact timeframe for this project to reach a usable state, however planning and work is beginning imminently.