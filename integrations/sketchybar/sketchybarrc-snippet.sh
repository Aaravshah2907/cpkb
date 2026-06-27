# CPKB Snippets
# Add this to your ~/.config/sketchybar/sketchybarrc after PLUGIN_DIR is set.

sketchybar --add item cpkb right \
           --set cpkb icon="</>" \
                      icon.font="JetBrainsMono Nerd Font:Bold:16.0" \
                      icon.color=0xffffffff \
                      label.drawing=off \
                      script="$PLUGIN_DIR/hover.sh $PLUGIN_DIR/cpkb_plugin.sh" \
           --subscribe cpkb mouse.clicked mouse.entered mouse.exited

# Optional: include cpkb in your existing reorder line.
# Example:
# sketchybar --reorder music volume battery cpkb clock
