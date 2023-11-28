# Palantir

Palantir is a Work In Progress P2P networking library written in Rust. 

At the time of viewing, there may not be any code published yet. I commonly keep Work-In-Progress libraries private until they are ready for release, however for this one I want to get to an MVP as fast as possible. While this is currently published with no code at version 0.0.0, there will be many sudden very breaking changes. For this purpose, until version 1.0.0 is released, every minor version can be considered to contain breaking changes. This is the same for other projects including [Fluxion](https://github.com/peperworx/fluxion), and a library to integrate this one with Fluxion (that is still in progress).

## Palantir's Expected Design

Peer to Peer networking is complicated, but Peer to Peer libraries don't have to be. While existing libraries are very robust and extremely extensible, they tend to have rather complicated interfaces and sometimes feel more like frameworks than libraries. Palantir will be designed to be very simple, and will not make any decisions for you on your application's design, excepting a few dependencies. Initializing a Palantir instance will look as follows:
1. Define your Transports
2. Describe your Network
3. Create/Retrieve an Identity
4. Connect to the Network using a Palantir instance
5. Get notified of new peers and
6. Send raw data to peers, and receive responses back.

Palantir will be starting with a very simple network implementation using a relay server. While a simple design like a relay server is not quite peer-to-peer, it will help lay the framework for Palantir's API. WASM support is expected in the future, however it may take some time. No-Std support is not a primary goal, however if any good solutions for executor and platform agnostic IO come up, then it is a possiblity.

## Why Is This Being Made?

I am making Palantir primarily for use with Fluxion. Fluxion enables messages to be sent between different actor systems. Palantir will enable sending messages to different computers on different networks without (much) backend infrastructure. I hope that this will pair well with Fluxion for building peer-to-peer applications.


It should be noted that there is no exact timeframe for this project to reach a usable state, however planning and work is beginning imminently.