# Goal
This project's aim originally was to implement an efficient networking client for the [OpenSimulator](http://opensimulator.org/wiki/Main_Page) network protocol in Rust. The goal was to provide a client of high enough quality to implement a new viewer on top of it that matches or surpasses the performance of current viewers.

dsrich is building a standalone asset server first, and is planning on using it as a platform to test the various proposals documented on the opensimulator.org web site.

## Compiler version
Before dsrich forked it, you needed nightly Rust for this, as leoschwartz decided that using `futures-await` was going to save development time. dsrich has not confirmed this need.

## Fork
This is forked from leoschwarz/opensim-networking since leoschwarz archived his repository. Nothing had happened for four years or so before that.

## Documentation
As the documentation of the protocol is rather sparse and this library is still not that far, consider this an early work in progress. There are multiple coexisting "protocols" so following is a list of them and the respective status of their implementation in this library.

## Status as of dsrich's fork

### Implemented:

- UDP messages: Handling of acks works fine. More debugging utilities will have to be added because for viewer development it will most likely be needed.
- Login protocol: Will need some more refinement and better error handling, but it's enough for testing purposes.

### Currently being worked on:

- Texture download
- Region download

### Soon to be worked on:

- Prims
- Mesh data

### Backlog:

- Sound
- Voice
- Inventory

## Protocol

The main goal of this library is to stay compatible with current versions of OpenSimulator. Since Second Life has changed their protocol, this library will most likely never be usable with their servers.

leoschwartz was in the process of collecting as much documentation on the protocol as possible in order to write a good and correct client for it. Many pieces of information are found across the internet and in various sources, so he was collecting his information in the repo [opensim-protocol](https://github.com/dsrich/opensim-protocol) which I have also forked. Ideally it should be an exact specification of the network protocol implemented by this client.
