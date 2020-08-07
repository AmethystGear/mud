import sys
import socket
import json
import colors
from enum import Enum
from threading import Thread
from collections import deque

DEFAULT_PORT = 31415
MAX_COMMAND_NAME_LEN = 5
MAX_COMMAND_SIZE_LEN = 5

init_data = None

def find_nth_char(st, ch, n):
    count = 0
    idx = 0
    for c in st:
        if chr(c) == ch:
            count += 1
            if count > n:
                return idx
        
        idx += 1
    
    return -1

class Commands(Enum):
    Text = 0
    Img = 1
    Init = 2

class Player:
    def __init__(self, ID, x, y):
        self.ID = ID
        self.x = x
        self.y = y
    
    def __repr__(self):
        return "(" + str(self.ID) + " " + str(self.x) + " " + str(self.y) + ")"

class PacketIncomplete(Exception):
    pass

class PacketBroken(Exception):
    pass

class Packet:
    def __init__(self, command, content):
        self.command = command
        self.content = content

def get_packet(msg):
    first_delim = find_nth_char(msg, ':', 0)
    second_delim = find_nth_char(msg, ':', 1)
    if first_delim == -1 and len(msg) > MAX_COMMAND_NAME_LEN:
        raise PacketBroken()
    elif first_delim == -1:
        raise PacketIncomplete()

    if second_delim == -1 and (len(msg) - first_delim) > MAX_COMMAND_SIZE_LEN:
        raise PacketBroken()
    elif second_delim == -1:
        raise PacketIncomplete()
    
    command = msg[0:first_delim].decode('ascii')
    if not command in Commands._member_names_:
        raise PacketBroken()

    try:
        msg_len = int(msg[(first_delim + 1):second_delim].decode('ascii'))
    except ValueError:
        raise PacketBroken()

    content = msg[(second_delim + 1):]
    if len(content) < msg_len:
        raise PacketIncomplete()
    else:
        pkt = Packet(Commands[command], content[0:msg_len])
        extra = content[msg_len:]
        return (pkt, extra)

def get_int_16_arr_from_bytes(bt):
    arr = deque()
    prev = None
    for b in bt:
        if prev is None:
            prev = b << 8
        else:
            arr.append(prev + b)
            prev = None
    
    return arr

def handle_packet(pkt):
    global init_data
    if pkt.command == Commands.Text:
        print(pkt.content.decode('utf-8'), end ='', flush=True)
    elif pkt.command == Commands.Img:
        ints = get_int_16_arr_from_bytes(pkt.content)
        if init_data is None:
            raise RuntimeError('init data not initialized!')

        default_entity_string = init_data['default_entity']
        num_players = ints.popleft()
        players = {}
        for i in range(0, num_players):
            player = Player(ints.popleft(), ints.popleft(), ints.popleft())
            players[(player.y << 8) + player.x] = player

        width = ints.popleft()
        num_elems = ints.popleft()
        blocks = []
        for i in range(0, num_elems):
            blocks.append(ints.popleft())

        num_elems_ent = ints.popleft()
        entities = []
        for i in range(0, num_elems_ent):
            entities.append(ints.popleft())
        
        
        for y in range(0, int(num_elems/width)):
            line = ''
            sub_line = ''
            prev = blocks[y * width]
            for x in range(0, width):
                index = y * width + x
                
                display = '  '
                if ((y << 8) + x) in players:
                    display = str(players[((y << 8) + x)].ID) * 2
                elif (len(entities) != 0 and entities[index] != 65535):
                    if str(entities[index]) in init_data['entity_display']:
                        display = init_data['entity_display'][str(entities[index])]
                    else:
                        display = init_data['default_entity']

                if blocks[index] == prev:
                    sub_line += display
                else:
                    line += colors.color(sub_line, bg=init_data['block_display'][str(prev)])
                    sub_line = display
                
                prev = blocks[index]

            line += colors.color(sub_line, bg=init_data['block_display'][str(prev)])
            print(line)

    elif pkt.command == Commands.Init:
        init_data = json.loads(pkt.content.decode('utf-8'))

def recv(s):
    msg = bytearray()
    while True:
        data = s.recv(1024)
        msg.extend(data)
        while True:
            try:
                pkt, extra = get_packet(msg)
                handle_packet(pkt)
                msg = extra
            except PacketIncomplete:
                break

            except PacketBroken:
                print("ERROR: recieved badly formatted packet:\n" + str(msg))
                print("continuing")
                msg = bytearray()
                continue

def main():
    if len(sys.argv) == 1 or len(sys.argv) > 3:
        print("ERROR: expected ip address, or ip address and port!")
        print("exiting")
        exit()

    addr = sys.argv[1]
    port = None
    if len(sys.argv) == 3:
        try:
            port = int(sys.argv[2])
        except ValueError:
            print("ERROR: port was invalid!")
            print("exiting")
            exit()
    else:
        port = DEFAULT_PORT

    print("Connecting to address " + addr + " at port " + str(port))
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((addr, port))

    Thread(target=recv, args=(s,)).start()
    while (True):
        command = input() + '\n'
        s.sendall(command.encode('utf-8'))

main()