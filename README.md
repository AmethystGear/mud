# mud
A terminal game where you explore a randomly generated world and fight monsters!

mud is 100% text based, but it uses 256-color (https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit) to display 'images' by printing out sequences of characters.
If your terminal doesn't support 256-color, then either get another terminal, or alternatively download vscode, which comes with a dedicated terminal that supports 256-color. vscode should work for mac, linux, and windows for the purposes of this game.

## using mud
All the following commands should be run from the main directory.

## requirements:
If you're on mac/linux, you just need java. If you're on windows, you need java and python 3.

### compiling the server

if you're using mac/linux, you can use wildcard paths to compile the server:
```javac server/*/*.java```

if you're using windows, wildcards won't work as expected. Instead there is a small python script you can use. To run it, do:
```python3 compile_server.py```

### running the server
If you're on mac/linux, do:
```java server/main/MudServer```

If you're on windows, it's the same except you need to change the slashes:
```java server\main\MudServer```

### running the client
There's only one file for the client, so no need to do any compilation.

If you're on mac/linux, do:
```java server/main/MudClient.java```

If you're on windows, it's the same except you need to change the slashes:
```java server\main\MudClient.java```
