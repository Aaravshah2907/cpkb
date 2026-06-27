#!/usr/bin/env bash

# Small helper used by the example SketchyBar item. It animates hover state,
# then delegates to the plugin passed as arguments.

NORMAL_SIZE=18
HOVER_SIZE=24

if [[ "$SENDER" == "mouse.entered" ]]; then
  sketchybar --animate tanh 15 --set "$NAME" icon.font.size=$HOVER_SIZE
elif [[ "$SENDER" == "mouse.exited" ]]; then
  sketchybar --animate tanh 15 --set "$NAME" icon.font.size=$NORMAL_SIZE
fi

if [[ -n "$1" ]]; then
  "$@"
fi
