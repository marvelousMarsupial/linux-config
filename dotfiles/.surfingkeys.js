// Edit this repo file, copy it with scripts/install, then import it in
// Surfingkeys' advanced settings. See docs/browser.md for the test checklist.

// Return to Normal mode without reaching for Escape.
// Normal-mode j keeps its default scroll behavior.
api.iunmap('jj');
api.imap('jj', '<Esc>');
api.vmap('jj', '<Esc>');
api.cmap('jj', '<Esc>');
api.aceVimMap('jj', '<Esc>', 'insert');

settings.aceKeybindings = 'vim';
settings.smoothScroll = true;
settings.omnibarPosition = 'middle';
settings.omnibarMaxResults = 10;

// Rounded Dracula palette, with an installed font and readable fallbacks.
settings.theme = `
.sk_theme {
    font-family: "CaskaydiaCove Nerd Font", "JetBrains Mono", Consolas, monospace;
    font-size: 14px;
    line-height: 1.5;
    background: #282a36;
    color: #f8f8f2;
}
.sk_theme input {
    font: inherit;
    color: #f8f8f2;
    caret-color: #bd93f9;
    background: transparent;
}
.sk_theme .url { color: #8be9fd; }
.sk_theme .annotation, .sk_theme .omnibar_timestamp,
.sk_theme .omnibar_visitcount { color: #b0b7d1; }
.sk_theme .omnibar_folder, .sk_theme .feature_name { color: #bd93f9; }
.sk_theme .omnibar_highlight { color: #50fa7b; font-weight: 700; }
.sk_theme .prompt, .sk_theme .resultPage { color: #bd93f9; }
.sk_theme .separator { color: #6272a4; }
.sk_theme kbd {
    background: #44475a;
    color: #f1fa8c;
    border: 1px solid #6272a4;
    border-radius: 5px;
    padding: 2px 5px;
    box-shadow: none;
}
#sk_omnibar {
    box-sizing: border-box;
    width: min(900px, 92vw);
    left: 50%;
    transform: translateX(-50%);
    background: #282a36;
    border: 1px solid #6272a4;
    border-radius: 14px;
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.45);
    overflow: hidden;
}
#sk_omnibarSearchArea, .sk_omnibar_middle #sk_omnibarSearchArea {
    margin: 0;
    padding: 14px 18px;
    gap: 10px;
    background: #303341;
    border-bottom: 1px solid #44475a;
}
#sk_omnibarSearchArea > input {
    min-width: 0;
    font-size: 16px;
    padding: 0;
}
#sk_omnibarSearchArea .prompt, #sk_omnibarSearchArea .resultPage {
    font-size: 14px;
}
#sk_omnibarSearchResult {
    margin: 6px;
    max-height: 58vh;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #6272a4 #282a36;
}
#sk_omnibarSearchResult > ul { margin: 0; }
.sk_theme #sk_omnibarSearchResult > ul > li {
    padding: 8px 12px;
    margin: 3px 0;
    border-radius: 8px;
    border-left: 3px solid transparent;
}
.sk_theme #sk_omnibarSearchResult > ul > li:nth-child(odd) {
    background: #303341;
}
.sk_theme #sk_omnibarSearchResult > ul > li.focused {
    background: #44475a;
    border-left-color: #bd93f9;
}
#sk_status, #sk_keystroke, #sk_banner, #sk_usage, #sk_popup, #sk_editor {
    background: #282a36;
    color: #f8f8f2;
    border: 1px solid #6272a4;
    border-radius: 10px;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.35);
}
#sk_status, #sk_keystroke { padding: 8px 12px; font-size: 14px; }
#sk_find { background: transparent; color: #f8f8f2; font: inherit; }
#sk_usage, #sk_popup { padding: 16px; }
#sk_frame { border-radius: 10px; }

/* sk_editor itself is the ACE editor; let ACE manage its internal layout. */
#sk_editor {
    box-sizing: border-box;
    width: min(1200px, 92vw) !important;
    height: 80vh !important;
    max-height: 80vh !important;
    top: 10vh !important;
    left: 50% !important;
    transform: translateX(-50%);
    font-size: 14px;
}
`;

const hintStyle = [
    'border: 1px solid #bd93f9',
    'color: #282a36',
    'background: #f1fa8c',
    'font-family: "CaskaydiaCove Nerd Font", "JetBrains Mono", monospace',
    'font-size: 13px',
    'font-weight: 700',
    'padding: 2px 5px',
    'border-radius: 5px',
    'box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35)',
].join('; ');
api.Hints.style(hintStyle);
api.Hints.style(hintStyle, 'text');
