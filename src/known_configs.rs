/// List of known configuration directories that YAAT will auto-detect
pub const KNOWN_CONFIGS: &[&str] = &[
    // Desktop/Window Manager
    "hypr",
    "waybar",
    "walker",
    "mako",
    "omarchy",
    "swaync",
    "dunst",
    "rofi",
    "wofi",
    // Shells
    "fish",
    "zsh",
    "bash",
    // Terminals
    "kitty",
    "alacritty",
    "wezterm",
    "ghostty",
    "foot",
    // Editors
    "nvim",
    "vim",
    "emacs",
    "helix",
    // Terminal multiplexers
    "tmux",
    "tmuxinator",
    "zellij",
    // Git
    "git",
    "lazygit",
    // Dotfiles tools
    "yadm",
    "chezmoi",
    "stow",
    // Development tools
    "mise",
    "asdf",
    "nvm",
    "pyenv",
    // File managers
    "ranger",
    "lf",
    "nnn",
    "yazi",
    // Media
    "mpd",
    "ncmpcpp",
    "mpv",
    // System tools
    "btop",
    "htop",
    "neofetch",
    "fastfetch",
    "direnv",
    "fzf",
    "zoxide",
    "starship",
    "oh-my-posh",
    // Input methods
    "fcitx",
    "fcitx5",
    "ibus",
];

/// Common home directory files to check
pub const KNOWN_HOME_FILES: &[(&str, &str)] = &[
    (".tmux.conf", "home/.tmux.conf"),
    (".gitconfig", "home/.gitconfig"),
    (".bashrc", "home/.bashrc"),
    (".zshrc", "home/.zshrc"),
    (".vimrc", "home/.vimrc"),
];
