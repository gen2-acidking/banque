# Bank

![Banque](banque.png)

Command storage and execution tool. Store frequently used commands and run them by index.

---

## Install
?
Choose your binary type:

```bash
# Dynamic (default)
curl -sSL https://github.com/gen2-acidking/banque/raw/master/install.sh | bash

# Static (musl(based))
curl -sSL https://github.com/gen2-acidking/banque/raw/master/install.sh | bash -s -- --static

# After install, run:
banque help
# or just
banque
```

---

## Uninstall

```bash
curl -sSL https://github.com/gen2-acidking/banque/raw/master/uninstall.sh | bash
```

---

## Features

- Command Storage: Save commands with auto or custom labels  
- Favorites: Mark frequently used commands as favorites  
- Quick Execution: Run commands by index number  
- Persistent: Commands stored in `~/.bank.txt`  
- Shell Integration: Works with bash and zsh  
- Static and Dynamic Binaries: Choose during install  
- One-Line Installer and Uninstaller  

---


---

## License

[MIT](LICENSE)
