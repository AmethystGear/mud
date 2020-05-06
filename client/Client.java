package client;

import java.io.*;
import java.net.*;

public class Client {
    public static void main(String[] args) throws Exception {
        BufferedReader inFromUser = new BufferedReader(new InputStreamReader(System.in));
        System.out.print("Enter the ip address you want to connect to: ");
        String ipAddrStr = inFromUser.readLine();
        InetAddress addr = null;
        boolean unknownHost = true;
        while (unknownHost) {
            try {
                addr = Inet4Address.getByName(ipAddrStr);
                unknownHost = false;
            } catch (UnknownHostException e) {
                System.out.println("That host doesn't exist! Try a different host.");
                System.out.print("Enter the ip address you want to connect to: ");
                ipAddrStr = inFromUser.readLine();
            }
        }

        int port = -1;
        System.out.print("Enter a port: ");
        String portStr = inFromUser.readLine();
        boolean notParseable = true;
        while (notParseable) {
            try {
                port = Integer.parseInt(portStr);
                if (port > 65535 || port < 0) {
                    throw new NumberFormatException();
                } else {
                    notParseable = false;
                }
            } catch (NumberFormatException e) {
                System.out.println("That port was invalid.");
                System.out.print("Enter a port: ");
                portStr = inFromUser.readLine();
            }
        }

        Socket clientSocket = new Socket(addr, port);
        DataOutputStream outToServer = new DataOutputStream(clientSocket.getOutputStream());
        BufferedReader inFromServer = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));

        ServerPrinter s = new ServerPrinter(inFromServer);
        s.start();

        while (true) {
            System.out.print("Enter a command: ");
            String command = inFromUser.readLine();
            if (command.equals("quit")) {
                clientSocket.close();
                outToServer.close();
                break;
            } else {
                outToServer.writeBytes(command + '\n');
            }
        }

        s.exit();
    }

    // this class prints everything it recieves from the server.
    private static class ServerPrinter extends Thread {
        private volatile boolean exit = false;
        BufferedReader inFromServer;

        public ServerPrinter (BufferedReader inFromServer) {
            this.inFromServer = inFromServer;
        }

        public void exit() throws IOException {
            inFromServer.close();
            exit = true;
        }

        public void run() {
            while(!exit) {
                String line;
                try {
                    line = inFromServer.readLine();
                } catch (IOException e) {
                    continue;
                }
                boolean first = true;
                while(!line.contains("/end/")) {
                    if(first) {
                        int index = line.indexOf("/begin/");
                        if(index != -1) {
                            System.out.println(line.substring(index + 7, line.length()));
                            first = false;
                        }
                    } else {
                        System.out.println(line);
                    }
                    try {
                        line = inFromServer.readLine();
                    } catch(IOException e) {
                        break;
                    }
                }
            }
        }
    }
}
