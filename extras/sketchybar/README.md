# SketchyBar Integration

This optional macOS integration adds a CPKB item to SketchyBar. Clicking it opens a search dialog, lists matching snippets, and copies the selected snippet with `cpkb copy`.

## Requirements

- macOS
- SketchyBar
- CPKB installed and available on `PATH`
- A Nerd Font if you want the example font styling

## Install

Copy the plugin files into your SketchyBar plugins directory:

```bash
cp integrations/sketchybar/cpkb_plugin.sh ~/.config/sketchybar/plugins/
cp integrations/sketchybar/hover.sh ~/.config/sketchybar/plugins/
chmod +x ~/.config/sketchybar/plugins/cpkb_plugin.sh
chmod +x ~/.config/sketchybar/plugins/hover.sh
```

Add the contents of `sketchybarrc-snippet.sh` to your `~/.config/sketchybar/sketchybarrc` after `PLUGIN_DIR` is defined.

Then reload SketchyBar:

```bash
sketchybar --reload
```

## Customize

The snippet uses a plain white icon color:

```bash
icon.color=0xffffffff
```

Replace that with one of your theme variables if your SketchyBar config defines colors, for example:

```bash
icon.color=$FRONTAPP_ACCENT
```
