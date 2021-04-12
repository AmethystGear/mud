// all the potential types of Packets we can expect.
const PacketTypes = Object.freeze({ "Text": 0, "Img": 1, "Init": 2, "Err": 3 })

const MAX_PACKET_TYPE_LEN = 5
const MAX_PACKET_SIZE_LEN = 4
const MAX_TEXTAREA_SIZE = 25000

let socket = null
let canvas = null
let ct = null
let text = null
let textbox = null
let initData = null
let imgData = null
let buffer = null
let lastSent = ""

class Player {
    constructor(ID, x, y) {
        this.ID = ID
        this.x = x
        this.y = y
    }

    /**
     * get a string representing the player
     * @returns {string} - player string representation
     */
    getPlayerDisplay() {
        let hex = Number(this.ID).toString(16);
        if (hex.length == 1) {
            hex = "0" + hex
        }
        return hex
    }

    /**
     * get hash of player's position
     * @returns {string} - hash of player's position
     */
    getPosnHash() {
        return posnHash(this.x, this.y)
    }
}

function posnHash(x, y) {
    return "" + ((y << 8) + x)
}

class Packet {
    constructor(packetType, content) {
        this.packetType = packetType
        this.content = content
    }
}

class Uint8Iter {
    constructor(data) {
        this.data = data
        this.index = 0
    }

    pop() {
        let val = this.data.getUint8(this.index, false)
        this.index++
        return val
    }
}

/** custom error thrown when packet is incomplete (needs more bytes) */
class PacketIncomplete extends Error {
    constructor(message) {
        super(message)
        this.name = "PacketIncompleteError"
    }
}

/** custom error thrown when packet is broken (badly formatted) */
class PacketBroken extends Error {
    constructor(message) {
        super(message)
        this.name = "PacketBrokenError"
    }
}

/** custom error thrown when a packet requires init data, 
    but init data has not been recieved */
class InitDataNotInitialized extends Error {
    constructor(message) {
        super(message)
        this.name = "InitDataNotInitializedError"
    }
}

/**
 * finds the index of the Nth ascii char in a uint8array
 * @param {Uint8Array} st - array in which we look for ascii char
 * @param {string} ch - string containing a single ascii char
 * @param {int} n - specify the Nth char we are looking for
 * @returns {int} - index of the Nth char, or -1 if not found
 */
function findNthCharIndex(st, ch, n) {
    let ascii = ch.charCodeAt(0);
    let count = 0
    for (let i = 0; i < st.length; i++) {
        if (st[i] === ascii) {
            if (count === n) {
                return i
            }
            count++;
        }
    }
    return -1
}

/**
 * gets first packet from Uint8Array buffer
 * @param {Uint8Array} msg - buffer of bytes
 * @throws {PacketIncomplete} - if needs more bytes to construct packet
 * @throws {PacketBroken} - if packet is badly formatted and needs to be thrown out
 * @returns {{Packet}, {Uint8Array}} - packet, and remaining unused bytes
 */
function getPacket(msg) {
    let delims = [-1, -1]
    let maxes = [MAX_PACKET_TYPE_LEN, MAX_PACKET_SIZE_LEN]
    for (let i = 0; i < delims.length; i++) {
        delims[i] = findNthCharIndex(msg, ':', i)
        if (delims[i] === -1 && msg.length > maxes[i]) {
            throw new PacketBroken()
        } else if (delims[i] === -1) {
            throw new PacketIncomplete()
        }
    }

    let dec = new TextDecoder()
    let packetTypeStr = dec.decode(msg.slice(0, delims[0]))
    let packetType = null;
    try {
        packetType = PacketTypes[packetTypeStr]
    } catch (err) {
        throw new PacketBroken()
    }

    let msgLen = null;
    try {
        msgLen = parseInt(dec.decode(msg.slice(delims[0] + 1, delims[1])))
    } catch (err) {
        throw new PacketBroken()
    }

    let content = msg.slice(delims[1] + 1, msg.length)
    if (content.length < msgLen) {
        throw new PacketIncomplete()
    }
    let pkt = new Packet(packetType, content.slice(0, msgLen))
    let extra = content.slice(msgLen, content.length)
    return { pkt: pkt, extra: extra }
}

/**
 * does the action required by the packet
 * @param {Packet} pkt - packet of information recieved from server
 */
function handlePacket(pkt) {
    let dec = new TextDecoder()
    if (pkt.packetType === PacketTypes.Text) {
        let decoded = dec.decode(pkt.content)
        displayString(decoded)
    } else if (pkt.packetType === PacketTypes.Img) {
        if (initData === null) {
            throw new InitDataNotInitialized()
        }
        displayImg(new Uint8Iter(new DataView(pkt.content.buffer)))
    } else if (pkt.packetType === PacketTypes.Init) {
        initData = JSON.parse(dec.decode(pkt.content))
        imgData = {}
        for (var entity in initData) {
            let name = initData[entity]
            imgData[name] = new Image()
            imgData[name].src = "resources/images/" + name
        }
    } else if (pkt.packetType === PacketTypes.Err) {
        let decoded = dec.decode(pkt.content)
        let err = "ERROR: " + decoded + '\n'
        displayString(err)
    }
}

/**
 * display string on the webpage
 * @param {string} string - string to display
 */
function displayString(string) {
    text.value = text.value + string
    if (text.value.length > MAX_TEXTAREA_SIZE) {
        text.value = text.value.substring(text.value.length * 0.3, text.value.length)
    }
    text.scrollTop = text.scrollHeight
}

/**
 * display image on the webpage
 * @param {Uint8Iter} data
 */
function displayImg(data) {
    let width = data.pop()
    let height = data.pop()
    let numPlayers = data.pop()
    let numElems = width * height
    let numElemsEnt = data.pop()

    let players = []
    for (let i = 0; i < numPlayers; i++) {
        let player = new Player(data.pop(), data.pop(), data.pop())
        players.push(player)
    }

    let blocks = []
    for (let i = 0; i < numElems; i++) {
        blocks.push([data.pop(), data.pop(), data.pop()])
    }

    let entities = []
    for (let i = 0; i < numElemsEnt; i++) {
        entities.push(data.pop())
    }
    let blockWidth = Math.round(canvas.width / width)
    let blockHeight = Math.round(canvas.height / (numElems / width))

    for (let y = 0; y < (numElems / width); y++) {
        let yPx = y * blockHeight
        for (let x = 0; x < width; x++) {
            let xPx = x * blockWidth
            let index = y * width + x
            let block = blocks[index]
            ct.fillStyle = "rgb(" + block[0] + "," + block[1] + "," + block[2] + ")";
            ct.fillRect(xPx, yPx, blockWidth, blockHeight)
        }
    }

    if (numElemsEnt > 0) {
        for (let y = 0; y < (numElems / width); y++) {
            let yPx = y * blockHeight
            for (let x = 0; x < width; x++) {
                let xPx = x * blockWidth
                let index = y * width + x
                let entity = entities[index]
                if (entity === 255) {
                    continue
                }
                let entityImg = initData[entity]
                ct.drawImage(imgData[entityImg], xPx, yPx, blockWidth, blockHeight)
            }
        }
    }

    ct.fillStyle = "#FF0000"
    ct.font = Math.round(blockWidth) + "px Consolas";
    for (let i = 0; i < players.length; i++) {
        ct.fillText(players[i].getPlayerDisplay(), players[i].x * blockWidth, (players[i].y + 1) * blockHeight)
    }
}

function quoted(str) {
    str.charAt(0) == '"' && str.charAt(str.length - 1) == '"'
}

function onTextboxEnter() {
    let content = textbox.value.trim().split(/[ ,]+/)
    let send = ""
    for (i = 0; i < content.length; i++) {
        if (!isNaN(content[i]) || content[i] == 'true' || content[i] == 'false') {
            send += content[i]
        } else {
            send += "\"" + content[i] + "\""
        }
        send += " "
    }
    send = send.trim()
    if (send != "") {
        socket.send("[" + send + "]")
        lastSent = send
    } else {
        socket.send("[" + lastSent + "]")
    }
    textbox.value = ""
}

function concatTypedArrays(a, b) {
    var c = new (a.constructor)(a.length + b.length);
    c.set(a, 0);
    c.set(b, a.length);
    return c;
}

$(document).ready(function () {
    (async () => {
        let ws = await fetch('/ws-server-loc').then(function (response) { return response.json() })
        socket = new WebSocket("ws://" + ws['ip'] + ":" + ws['port'])
        canvas = document.getElementById("canvas")
        ct = canvas.getContext("2d");
        text = document.getElementById("text")
        textbox = document.getElementById("textbox")
        initData = null
        buffer = new Uint8Array(0)
        text.scrollTop = text.scrollHeight;

        textbox.addEventListener('keydown', (e) => {
            if (e.key == 'Enter') {
                onTextboxEnter()
            }
        });

        // auto display (10fps)
        window.setInterval(function() {
            if (lastSent != "\"map\"") {
                socket.send("[\"disp\"]")
            }
        }, 100);

        socket.binaryType = 'arraybuffer';
        socket.onmessage = function (evt) {
            let data = new Uint8Array(evt.data)
            buffer = concatTypedArrays(buffer, data)

            while (true) {
                try {
                    const { pkt, extra } = getPacket(buffer)
                    handlePacket(pkt)
                    buffer = extra
                } catch (err) {
                    if (err instanceof PacketIncomplete) {
                        break
                    } else if (err instanceof PacketBroken) {
                        buffer = new Uint8Array(0)
                        break
                    } else if (err instanceof InitDataNotInitialized) {
                        buffer = new Uint8Array(0)
                        break
                    } else {
                        throw err
                    }
                }
            }

        };
    })();
});


