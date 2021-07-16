# mirae
A terminal-based multiplayer game where you explore a randomly generated world and fight monsters!
This takes inspiration from the original Rougelike games, but uses a more open world setting.
## using mirae
All the following commands should be run from the main directory.

## requirements:
if you want to run the server, you need rust. If you want to run the client, you need python.

### compiling the server
do:
```cargo build release```
in the `mirae_server/` folder.

### running the server
do:
```cargo run```
in the `mirae_server/` folder.

### running the client
just one python file.
do:```python client/client.py <ip-address> <port (if you don't want the default port)>```

the client should look something like this if you're doing it right:
![image](https://user-images.githubusercontent.com/21998904/81458324-a8947680-914e-11ea-9215-0d2817299ca9.png)
try typing `help` into the client end if you're not sure of what to do!
