#!/bin/bash

pkill -f server-earth
pkill -f server-mars

# build everything
cargo build

# run the servers
cargo run -p server-earth &> /dev/null &
cargo run -p server-mars &> /dev/null &

# sleep for a bit to let the servers start
sleep 2

# run the engine
cargo run -p engine
