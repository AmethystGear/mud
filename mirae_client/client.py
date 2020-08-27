import http.server
import socketserver
import requests
import json
import sys

CLIENT_PORT = int(sys.argv[1])
WS_SERVER_PORT = int(sys.argv[2])
IP_SERVICE = 'https://api.ipify.org'

if len(sys.argv) == 4:
    PUB_IP = sys.argv[3]
else:
    PUB_IP = requests.get(IP_SERVICE).text

class Server(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        global PUB_IP
        if self.path == "/ws-server-loc":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            data = json.dumps({'ip': str(PUB_IP), 'port': WS_SERVER_PORT})
            self.wfile.write(bytes(data, 'utf-8'))
        else:
            http.server.SimpleHTTPRequestHandler.do_GET(self)

with socketserver.TCPServer(("", CLIENT_PORT), Server) as httpd:
    print("serving at port", CLIENT_PORT)
    httpd.serve_forever()
