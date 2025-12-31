# PATH
export PATH="$HOME/.local/bin:$PATH"
export VIMRUNTIME="$HOME/.local/nvim-runtime/usr/share/nvim/runtime"
# Base16 shell colors
[ -f ~/.local/share/base16-shell/profile_helper.sh ] && source ~/.local/share/base16-shell/profile_helper.sh

# Plugins
source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

# Starship prompt
eval "$(starship init zsh)"

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
