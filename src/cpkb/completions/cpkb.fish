function __cpkb_needs_command
    set cmd (commandline -opc)
    if test (count $cmd) -eq 1
        return 0
    end
    return 1
end

function __cpkb_using_command
    set cmd (commandline -opc)
    if test (count $cmd) -gt 1
        if test $cmd[2] = $argv[1]
            return 0
        end
    end
    return 1
end

function __cpkb_get_snippets
    cpkb query "" --limit 100 2>/dev/null | while read -l line
        echo $line | awk -F '|' '{gsub(/^[ \t]+|[ \t]+$/, "", $1); gsub(/^[ \t]+|[ \t]+$/, "", $2); print $1 "\t" $2}'
    end
end

complete -f -c cpkb -n '__cpkb_needs_command' -a add -d 'Add a new snippet'
complete -f -c cpkb -n '__cpkb_needs_command' -a edit -d 'Edit an existing snippet'
complete -f -c cpkb -n '__cpkb_needs_command' -a delete -d 'Delete a snippet'
complete -f -c cpkb -n '__cpkb_needs_command' -a list -d 'List all snippets'
complete -f -c cpkb -n '__cpkb_needs_command' -a recent -d 'Show recent snippets'
complete -f -c cpkb -n '__cpkb_needs_command' -a show -d 'Show snippet details'
complete -f -c cpkb -n '__cpkb_needs_command' -a search -d 'Search snippets'
complete -f -c cpkb -n '__cpkb_needs_command' -a query -d 'Scripting-friendly query command'
complete -f -c cpkb -n '__cpkb_needs_command' -a use -d 'Record a usage of a snippet'
complete -f -c cpkb -n '__cpkb_needs_command' -a usages -d 'List usages of a snippet'
complete -f -c cpkb -n '__cpkb_needs_command' -a edit-usage -d 'Edit a usage record'
complete -f -c cpkb -n '__cpkb_needs_command' -a tag-add -d 'Add a tag to a snippet'
complete -f -c cpkb -n '__cpkb_needs_command' -a tag-remove -d 'Remove a tag from a snippet'
complete -f -c cpkb -n '__cpkb_needs_command' -a export -d 'Export all snippets to markdown'
complete -f -c cpkb -n '__cpkb_needs_command' -a export-json -d 'Export snippets to JSON'
complete -f -c cpkb -n '__cpkb_needs_command' -a export-html -d 'Export snippets to HTML'
complete -f -c cpkb -n '__cpkb_needs_command' -a export-db -d 'Export the SQLite database'
complete -f -c cpkb -n '__cpkb_needs_command' -a import -d 'Import snippets'
complete -f -c cpkb -n '__cpkb_needs_command' -a backup -d 'Create a manual backup'
complete -f -c cpkb -n '__cpkb_needs_command' -a config -d 'Show active configuration'
complete -f -c cpkb -n '__cpkb_needs_command' -a id-format -d 'Manage configured snippet ID formats'
complete -f -c cpkb -n '__cpkb_needs_command' -a setup -d 'Set up CPKB directories'
complete -f -c cpkb -n '__cpkb_needs_command' -a encrypt-db -d 'Encrypt the database'
complete -f -c cpkb -n '__cpkb_needs_command' -a decrypt-db -d 'Decrypt the database'
complete -f -c cpkb -n '__cpkb_needs_command' -a sync -d 'Sync database to Git remote'
complete -f -c cpkb -n '__cpkb_needs_command' -a tui -d 'Launch the Textual TUI'
complete -f -c cpkb -n '__cpkb_needs_command' -a fzf -d 'Search snippets using fzf'
complete -f -c cpkb -n '__cpkb_needs_command' -a copy -d 'Copy a snippet to the clipboard'
complete -f -c cpkb -n '__cpkb_needs_command' -a revise -d 'Spaced-repetition revision'
complete -f -c cpkb -n '__cpkb_needs_command' -a srs-stats -d 'Show spaced-repetition statistics'

# Complete IDs for relevant commands
for cmd in edit delete show use usages tag-add tag-remove copy
    complete -f -c cpkb -n "__cpkb_using_command $cmd" -a "(__cpkb_get_snippets)"
end
