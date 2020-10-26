
# Enabling in NeoVim + Coc

Start the server...

```
RUST_LOG="trace" cargo run
```

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

Open a file and chnage the file type with `:set ft=puml`
