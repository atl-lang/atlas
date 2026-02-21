# Atlas LSP - Neovim Setup

Configure the Atlas Language Server with Neovim's built-in LSP client.

---

## Prerequisites

- Neovim 0.8+ with built-in LSP
- Atlas CLI installed: `atlas --version`

---

## Configuration

### Using nvim-lspconfig

Add to your `init.lua`:

```lua
require('lspconfig').atlas_lsp.setup{
  cmd = {'atlas', 'lsp'},
  filetypes = {'atlas', 'atl'},
  root_dir = require('lspconfig.util').root_pattern('atlas.toml', '.git'),
  settings = {}
}
```

### Manual Configuration

```lua
vim.api.nvim_create_autocmd('FileType', {
  pattern = {'atlas', 'atl'},
  callback = function()
    vim.lsp.start({
      name = 'atlas-lsp',
      cmd = {'atlas', 'lsp'},
      root_dir = vim.fs.dirname(vim.fs.find({'atlas.toml', '.git'}, { upward = true })[1]),
    })
  end,
})
```

---

## Key Bindings

Add to your config:

```lua
vim.api.nvim_create_autocmd('LspAttach', {
  callback = function(args)
    local opts = { buffer = args.buf }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, opts)
  end,
})
```

---

## Features

All Atlas LSP features work in Neovim:
- Hover information (`K`)
- Code actions (`<leader>ca`)
- Symbol navigation (`gd`, `gr`)
- Formatting (`<leader>f`)
- Completion (with nvim-cmp)
- Signature help (auto-triggered on `(` and `,`)
- Find all references (`gr`)
- Rename symbol (`<leader>rn`)
- Call hierarchy (via `:lua vim.lsp.buf.incoming_calls()`)
- Workspace symbols (`:lua vim.lsp.buf.workspace_symbol()`)
- Inlay hints (Neovim 0.10+): `:lua vim.lsp.inlay_hint.enable(0, true)`
- Diagnostics (shown in sign column and virtual text)

### Extended Key Bindings

```lua
vim.api.nvim_create_autocmd('LspAttach', {
  callback = function(args)
    local opts = { buffer = args.buf }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'gD', vim.lsp.buf.declaration, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, opts)
    vim.keymap.set('n', '<leader>ws', vim.lsp.buf.workspace_symbol, opts)
    vim.keymap.set('n', '<leader>ic', vim.lsp.buf.incoming_calls, opts)
    vim.keymap.set('n', '<leader>oc', vim.lsp.buf.outgoing_calls, opts)
  end,
})
```

---

## Troubleshooting

Check LSP status:
```vim
:LspInfo
```

View logs:
```bash
tail -f ~/.local/state/nvim/lsp.log
```

Restart LSP:
```vim
:LspRestart
```

---

## More Information

- [LSP Features](../lsp-features.md)
- [Troubleshooting](../lsp-troubleshooting.md)
