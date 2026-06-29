#compdef cpkb

_cpkb() {
    local -a commands
    commands=(
        'add:Add a new snippet'
        'edit:Edit an existing snippet'
        'delete:Delete a snippet'
        'list:List all snippets'
        'recent:Show recent snippets'
        'show:Show snippet details'
        'search:Search snippets'
        'query:Scripting-friendly query command'
        'use:Record a usage of a snippet'
        'usages:List usages of a snippet'
        'edit-usage:Edit a usage record'
        'tag-add:Add a tag to a snippet'
        'tag-remove:Remove a tag from a snippet'
        'export:Export all snippets to markdown'
        'export-json:Export snippets to JSON'
        'export-html:Export snippets to HTML'
        'export-db:Export the SQLite database'
        'import:Import snippets'
        'backup:Create a manual backup'
        'config:Show active configuration'
        'id-format:Manage configured snippet ID formats'
        'setup:Set up CPKB directories'
        'encrypt-db:Encrypt the database'
        'decrypt-db:Decrypt the database'
        'sync:Sync database to Git remote'
        'tui:Launch the Textual TUI'
        'fzf:Search snippets using fzf'
        'copy:Copy a snippet to the clipboard'
        'revise:Spaced-repetition revision'
        'srs-stats:Show spaced-repetition statistics'
    )

    _arguments \
        '1: :->command' \
        '*: :->args'

    case $state in
        command)
            _describe -t commands "cpkb commands" commands
            ;;
        args)
            case $words[2] in
                edit|delete|show|use|usages|tag-add|tag-remove|copy)
                    local -a snippet_ids
                    local query_result
                    query_result=$(cpkb query "" --limit 100 2>/dev/null)
                    while IFS='|' read -r id title; do
                        id=$(echo "$id" | xargs)
                        title=$(echo "$title" | xargs | tr ':' '-')
                        if [[ -n "$id" ]]; then
                            snippet_ids+=("$id:$title")
                        fi
                    done <<< "$query_result"
                    
                    _describe -t snippet_ids "snippets" snippet_ids
                    ;;
            esac
            ;;
    esac
}

if type compdef &>/dev/null; then
    compdef _cpkb cpkb
fi
