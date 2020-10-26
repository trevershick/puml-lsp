
# Enabling in NeoVim + Coc

Run `:CocConfig` and add ...

```
{
  "languageserver": {
    "myls": {
        "host": "0.0.0.0",
        "port": 3030,
        "filetypes": ["puml"]
    }
  },
  "codeLens.enable": true
}
```
