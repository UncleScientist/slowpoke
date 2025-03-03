#!/bin/bash

ui="$1"
example="$2"

if [ -z "$ui" -o -z "$example" ]
then
    echo $0 '<ui>' '<example name>'
    exit 1
fi

if [ ! -d "crates/slowpoke-$ui" ]
then
    echo Unknown ui $ui
    exit 1
fi

if [ ! -f "examples/${example}.rs" ]
then
    echo Unknown example $example
    exit 1
fi

if [ ! -d "crates/slowpoke-$ui/examples" ]
then
    mkdir "crates/slowpoke-$ui/examples" || exit 1
fi

sed "s/use slowpoke::/use slowpoke_${ui}::/" \
    < "examples/${example}.rs" > "crates/slowpoke-$ui/examples/${example}.rs" || exit 1
cd "crates/slowpoke-$ui"
cargo run --example $example
