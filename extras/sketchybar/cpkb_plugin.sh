#!/usr/bin/env bash

# Optional SketchyBar plugin for CPKB.
# Click the item, search snippets, choose a result, and copy it to the clipboard.

if [ "$SENDER" = "mouse.clicked" ]; then
  SEARCH_TERM=$(osascript \
    -e 'Tell application "System Events" to display dialog "Search snippets:" default answer ""' \
    -e 'text returned of result' 2>/dev/null)

  if [ $? -eq 0 ]; then
    RESULTS=$(cpkb query "$SEARCH_TERM" --limit 5)

    if [ -z "$RESULTS" ]; then
      osascript -e 'display notification "No matching snippet found" with title "CPKB Search"'
      exit 0
    fi

    AS_LIST=""
    while read -r line; do
      if [ -n "$AS_LIST" ]; then
        AS_LIST="$AS_LIST, "
      fi
      line_escaped=$(echo "$line" | sed 's/"/\\"/g')
      AS_LIST="$AS_LIST\"$line_escaped\""
    done <<< "$RESULTS"

    SELECTION=$(osascript \
      -e "choose from list {$AS_LIST} with prompt \"Select a snippet to copy:\"" \
      2>/dev/null)

    if [ "$SELECTION" != "false" ] && [ -n "$SELECTION" ]; then
      SNIPPET_ID=$(echo "$SELECTION" | awk '{print $1}')
      cpkb copy "$SNIPPET_ID"
      osascript -e "display notification \"Snippet $SNIPPET_ID copied to clipboard!\" with title \"CPKB\""
    fi
  fi
fi
