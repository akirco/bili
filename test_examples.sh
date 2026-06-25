#!/usr/bin/env bash
cargo run --example login &&

ls -1 examples/ | sed 's/\.[^.]*$//' | grep -v login | xargs -I name cargo run --example name