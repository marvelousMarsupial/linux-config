# PATH - set BEFORE the sway auto-exec so sway and every child it spawns
# (.desktop files run by fuzzel, scripts called from binds, etc.) inherit
# ~/.local/bin and the sbin paths. Without this our binds hit "command not
# found" because Debian's default PATH for non-root users is bare bones.
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/local/sbin:/usr/sbin:/sbin:/usr/local/bin:/usr/bin:/sbin:/bin"

# Auto-start Sway on tty1
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ] && [ "$XDG_VTNR" = "1" ] && command -v sway >/dev/null; then
  exec sway
fi

# Locale for Unicode and emoji
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

export PASSWORD_STORE_DIR="$HOME/.password-store"

export PASSWORD_STORE_CLIP_CMD="wl-copy"
export GPG_TTY=$(tty)

# XDG runtime dir for desktop services
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

# D-Bus session (for headless sway)
# Import only the address; runtime files are data, never shell code.
if [[ -f "$XDG_RUNTIME_DIR/dbus-env" && -O "$XDG_RUNTIME_DIR/dbus-env" && ! -L "$XDG_RUNTIME_DIR/dbus-env" ]]; then
  while IFS= read -r _dbus_line; do
    if [[ "$_dbus_line" == 'export DBUS_SESSION_BUS_ADDRESS='* ]]; then
      _dbus_address="${_dbus_line#export DBUS_SESSION_BUS_ADDRESS=}"
      _dbus_address="${_dbus_address#\"}"
      _dbus_address="${_dbus_address%\"}"
      _dbus_address="${_dbus_address#\'}"
      _dbus_address="${_dbus_address%\'}"
      if [[ "$_dbus_address" == unix:* && "$_dbus_address" != *'$'* && "$_dbus_address" != *'`'* ]]; then
        export DBUS_SESSION_BUS_ADDRESS="$_dbus_address"
      fi
    fi
  done < "$XDG_RUNTIME_DIR/dbus-env"
  unset _dbus_line _dbus_address
fi

# Plugins
source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

# Starship prompt
eval "$(starship init zsh)"

# fnm (Fast Node Manager) shell integration. Loaded conditionally so
# the same .zshrc works with or without fnm installed.
# `command -v fnm` keeps initialization harmless when
# absent. `--use-on-cd` makes fnm auto-switch the active Node version
# when you `cd` into a directory containing a `.nvmrc` /
# `.node-version` file, which is the whole point of using fnm over a
# manual `nvm use`. The init also exports `FNM_MULTISHELL_PATH` and
# prepends it to `$PATH` (this happens AFTER our explicit PATH export
# at the top of the file, which is intentional — fnm's PATH manipulation
# needs to win for the active Node version to take precedence over
# any system Node).
[[ -d "$HOME/.local/share/fnm" ]] && export PATH="$HOME/.local/share/fnm:$PATH"
if command -v fnm >/dev/null; then
  eval "$(fnm env --use-on-cd --shell zsh)"
fi

# History
HISTFILE=~/.zsh_history
HISTSIZE=10000
SAVEHIST=10000
setopt appendhistory
setopt sharehistory
setopt hist_ignore_dups
setopt hist_ignore_space

# Yazi wrapper - cd to directory on exit
function y() {
  local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
  yazi "$@" --cwd-file="$tmp"
  if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
    builtin cd -- "$cwd"
  fi
  rm -f -- "$tmp"
}

# Source aliases
[[ -f ~/.zsh_aliases ]] && source ~/.zsh_aliases

# Alias reminder - inline terminal hints (opt-in)
# Shows dim "↳ gs" hint when you type "git status"
# Enable: export ALIAS_REMINDER=1
[[ -f ~/.zsh_alias_reminder ]] && source ~/.zsh_alias_reminder

# MOTD - show on interactive login
[[ -o login ]] && command -v motd >/dev/null && motd

# Foot shell integration: mark command output boundaries.
if [[ -o interactive && "$TERM" == foot* ]]; then
  autoload -Uz add-zsh-hook
  _foot_out_end()   { builtin zle || print -n '\e]133;D\e\\' }
  _foot_out_start() { print -n '\e]133;C\e\\' }
  add-zsh-hook precmd _foot_out_end
  add-zsh-hook preexec _foot_out_start
fi
