# Bash integration for heck. Source the output of: heck init bash
[[ $- == *i* ]] || return 0
# Refresh definitions on every source: an older integration may already be loaded.

_heck_previous_command() {
    local _heck_index _heck_input _heck_trimmed
    for ((_heck_index = -1; _heck_index >= -100; _heck_index--)); do
        _heck_input=$(builtin fc -ln "$_heck_index" "$_heck_index" 2>/dev/null) || return 1
        # Remove fc's formatting prefix, preserving the entry's own whitespace.
        _heck_input=${_heck_input#$'\t '}
        _heck_trimmed=${_heck_input#"${_heck_input%%[![:space:]]*}"}
        _heck_trimmed=${_heck_trimmed%"${_heck_trimmed##*[![:space:]]}"}
        if [[ -n $_heck_trimmed && $_heck_trimmed != heck ]]; then
            builtin printf '%s' "$_heck_input"
            return 0
        fi
    done
    return 1
}

_heck_pick() {
    local _heck_input=$1 _heck_program _heck_kind
    local -a _heck_args=()
    if [[ -z ${_heck_input//[[:space:]]/} ]]; then
        builtin printf 'heck: No command in recent shell history.\n' >&2
        return 1
    fi
    _heck_program=${_heck_input#"${_heck_input%%[![:space:]]*}"}
    _heck_program=${_heck_program%%[[:space:]]*}
    _heck_kind=$(builtin type -t -- "$_heck_program" 2>/dev/null) || _heck_kind=
    case $_heck_kind in
        alias|function) _heck_args+=(--program-shadowed) ;;
        file|builtin|keyword) _heck_args+=(--program-known) ;;
    esac
    builtin printf '%s' "$_heck_input" |
        command heck pick --stdin "${_heck_args[@]}"
}

_heck_bash_editor() {
    local _heck_edited
    # fc supplies its temporary editor file. Only a separate Enter at this
    # prefilled Readline prompt writes it back for fc to submit normally.
    IFS= builtin read -r -e -i "$_heck_selected" -p 'heck> ' _heck_edited </dev/tty ||
        return 130
    builtin printf '%s\n' "$_heck_edited" > "$1"
}

_heck_bash_command() {
    if (( $# )); then
        command heck "$@"
        return
    fi
    if [[ ! -t 0 || ! -t 1 ]]; then
        builtin printf "heck: Use 'heck suggest' for noninteractive input.\n" >&2
        return 2
    fi
    local _heck_input _heck_selected
    _heck_input=$(_heck_previous_command) || _heck_input=
    if _heck_selected=$(_heck_pick "$_heck_input"); then
        # fc handles the final submission and normal shell history behavior.
        builtin fc -e _heck_bash_editor -1
    else
        return $?
    fi
}

heck_uninstall() {
    if [[ -n ${_HECK_BASH_COMMAND_DEF-} &&
          $(builtin declare -f heck) == "$_HECK_BASH_COMMAND_DEF" ]]; then
        unset -f heck
    fi
    unset _HECK_BASH_COMMAND_DEF
    unset -f _heck_previous_command _heck_pick
    unset -f _heck_bash_editor _heck_bash_command heck_uninstall
}

_heck_kind=$(builtin type -t heck 2>/dev/null) || _heck_kind=
if [[ $_heck_kind == alias ||
      ( $_heck_kind == function &&
        $(builtin declare -f heck) != "${_HECK_BASH_COMMAND_DEF-}" ) ]]; then
    builtin printf 'heck: Keeping the existing heck alias/function. Remove or rename it to enable command repair.\n' >&2
else
    function heck { _heck_bash_command "$@"; }
    _HECK_BASH_COMMAND_DEF=$(builtin declare -f heck)
fi
unset _heck_kind
