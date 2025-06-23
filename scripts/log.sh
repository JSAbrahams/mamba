#!/bin/sh

# ANSI colors
export ANSI_GREEN='\033[0;32m'
export ANSI_RED='\033[0;31m'
export ANSI_YELLOW='\033[1;33m'
export ANSI_LIGHT_GREY='\033[1;37m'
export ANSY_CYAN='\033[0;36m'
export ANSI_NC='\033[0m' # No Color

_log() {
    level="${1}"
    shift

    newline="\n"
    message=""
    for arg in "$@"; do
        case "$arg" in
            -n)
                newline=""
                ;;
            *)
                message="$arg"
                ;;
        esac
    done

    printf "%b %s$newline" "$level" "$message"
}

# designed to be very permissive
# - if LOG_LEVEL undefined, then true (do log)
# - if LOG_LEVEL not a number, then true (do log)
# - if LOG_LEVLE a number, then check level and only log if > 1st argument
_log_level_null_or_nan_or_geq_than() {
    if [ -z "${LOG_LEVEL:-}" ]; then
        return 0
    fi

    case "$LOG_LEVEL" in
        *[!0-9]* | "")
            return 0
            ;;
        *)
            if [ "$LOG_LEVEL" -ge "$1" ]; then
                return 0
            else
                return 1
            fi
            ;;
    esac
}

is_log_lvl_error() {
    _log_level_null_or_nan_or_geq_than 1
}
log_error() {
    if is_log_lvl_error; then
        _log "${ANSI_RED}ERROR${ANSI_NC}" "$@"
    fi
}

is_log_lvl_warn() {
    _log_level_null_or_nan_or_geq_than 2
}
log_warn() {
    if is_log_lvl_warn; then
        # manually right justify
        _log " ${ANSI_YELLOW}WARN${ANSI_NC}" "$@"
    fi
}

is_log_lvl_info() {
    _log_level_null_or_nan_or_geq_than 3
}
log_info() {
    if is_log_lvl_info; then
        # manually right justify
        _log " ${ANSI_GREEN}INFO${ANSI_NC}" "$@"
    fi
}

is_log_lvl_debug() {
    _log_level_null_or_nan_or_geq_than 4
}
log_debug() {
    if is_log_lvl_debug; then
        _log "${ANSY_CYAN}DEBUG${ANSI_NC}" "$@"
    fi
}

is_log_lvl_trace() {
    _log_level_null_or_nan_or_geq_than 5
}
log_trace() {
    if is_log_lvl_trace; then
        _log "${ANSI_LIGHT_GREY}TRACE${ANSI_NC}" "$@"
    fi
}
