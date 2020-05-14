# mirae
A terminal-based multiplayer game where you explore a randomly generated world and fight monsters!
This takes inspiration from the original Rougelike games, but uses a more open world setting.

mirae is 100% text based, but it uses 256-color (https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit) to display 'images' by printing out sequences of characters.
If your terminal doesn't support 256-color, then either get another terminal, or alternatively download vscode, which comes with a dedicated terminal that supports 256-color. vscode should work for mac, linux, and windows for the purposes of this game.

## using mirae
All the following commands should be run from the main directory.

## requirements:
If you're on mac/linux, you just need java. If you're on windows, you need java and python 3.

### compiling the server

if you're using mac/linux, you can use wildcard paths to compile the server:
```javac server/*/*.java```

if you're using windows, wildcards won't work as expected. Instead there is a small python script you can use. To run it, do:
```python3 compile_server.py```

once you compile the server, you don't need to compile it again unless you delete any .class files or change the server code.
### running the server
do:```java server/main/Server```

### running the client
There's only one file for the client, so we can compile and run at the same time.

do:```java client/Client.java```

the client should look something like this if you're doing it right:
![image](https://user-images.githubusercontent.com/21998904/81458324-a8947680-914e-11ea-9215-0d2817299ca9.png)
try typing `help` into the client end if you're not sure of what to do!
