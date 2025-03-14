#!/bin/bash

cd ..
dfx build chain_fusion

forge build

if [ -d "$HOME/.cache/dfinity/pulled/7hfb6-caaaa-aaaar-qadga-cai" ] &&
   [ -d "$HOME/.cache/dfinity/pulled/ryjl3-tyaaa-aaaaa-aaaba-cai" ]
then
  echo "All remote canisters exist: skipping dfx pull."
else
 dfx deps pull
fi

ls -l .dfx/local/canisters/chain_fusion/* | grep "\(wasm\|service.did\$\)"
ls -l $HOME/.cache/dfinity/pulled/*

