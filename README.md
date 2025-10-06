# CAng

**CAng** is an interpreted programming language designed around resource constraints. In this language, you have a limited number of "coins" to spend on different features.
For example, you might start with 10 variable coins and 3 function coins. You can earn more coins by completing in-interpreter quests, such as "add 5 print calls" or "run a program 100 times."

## Demo (may take a few moments to load)

![Demo](/media/Screen%20Recording%202025-10-05%20at%203.57.10 PM.gif)

## Why?

Because I want siege coins. 🪙

## AI Usage

I used AI for:

* Generating Tests and Quests
* Debugging

## Build Instructions

### Dependencies

* Rust

### Steps

```sh
cargo build --release
./target/release/cang
# or
cargo run
# it's that simple
```

**Estimated AI Usage:** ~25% for everything.

## Running CAng

To install:

```sh
cargo install cang
```

Then, run `cang`.

I’ve adjusted the flow, made some grammar fixes, and polished the formatting. Does this work better for you?
