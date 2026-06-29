_cpkb_completions()
{
    local cur prev cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    cmds="add edit delete list recent show search query use usages edit-usage tag-add tag-remove export export-json export-html export-db import backup config id-format setup encrypt-db decrypt-db sync tui fzf copy revise srs-stats"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${cmds}" -- ${cur}) )
        return 0
    fi

    case "${prev}" in
        edit|delete|show|use|usages|tag-add|tag-remove|copy)
            # Fetch snippet IDs
            local query_result
            query_result=$(cpkb query "" --limit 100 2>/dev/null | awk -F '|' '{print $1}' | xargs)
            COMPREPLY=( $(compgen -W "${query_result}" -- ${cur}) )
            return 0
            ;;
    esac

    return 0
}

complete -F _cpkb_completions cpkb
