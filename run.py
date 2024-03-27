#!/bin/env python3

import os

os.system("cargo build --features bevy/dynamic_linking")
os.system("hyprctl dispatch workspace 3")
os.system("kitty --hold --detach sh -c 'cargo run --bin lobby-server --features bevy/dynamic_linking'")
os.system("kitty --hold --detach sh -c 'cargo run --bin client --features bevy/dynamic_linking'")