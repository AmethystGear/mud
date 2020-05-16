package client;

import java.io.*;
import java.net.*;

public class Client {

    private static final int MAX_CONNECTION_ATTEMPTS = 20;
    private static final int MAX_COMMAND_SEND_ATTEMPTS = 20;
    private static final int DEFAULT_PORT = 31415;
    private static final int MAX_PORT = 65535;
    private static final int MIN_PORT = 1024;

    public static void main(String[] args) {
        InetAddress addr = null;
        int port = -1;

        BufferedReader inFromUser = new BufferedReader(new InputStreamReader(System.in));
        if(args.length == 0) {
            boolean gotAddrAndIp = false;
            while (!gotAddrAndIp) {
                try {
                    addr = getIp(inFromUser);
                    port = getPort(inFromUser);
                    gotAddrAndIp = true;
                } catch (IOException e) {
                    // keep trying
                }
            }
        } else {
            if (args.length > 2) {
                System.out.println("ERROR: more than 2 arguments. " +
                                   "Was expecting either <ip address> <port>, or <ip address>.");
                return;
            }
            try {
                addr = Inet4Address.getByName(args[0]);
            } catch (UnknownHostException e) {
                System.out.println("ERROR: that ip address is invalid.");
                return;
            }
            if(args.length == 1) {
                port = DEFAULT_PORT;
            } else {
                try {
                    port = Integer.parseInt(args[1]);
                    if (port > MAX_PORT || port < MIN_PORT) {
                        System.out.println("Your port should be between " + MAX_PORT + " and " + MIN_PORT);
                        return;
                    }
                } catch(NumberFormatException e) {
                    System.out.println("That's not a number!");
                }
            }
        }

        Socket clientSocket = null;
        DataOutputStream outToServer = null;
        BufferedReader inFromServer = null;
        boolean setUpConnection = false;
        int connectionAttempts = 0;
        while(!setUpConnection) {
            try {
                clientSocket = new Socket(addr, port);
                outToServer = new DataOutputStream(clientSocket.getOutputStream());
                inFromServer = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));
                setUpConnection = true;
            } catch (IOException e) {
                e.printStackTrace();
                if(connectionAttempts == MAX_CONNECTION_ATTEMPTS) {
                    System.out.println("failed to connect after " + MAX_CONNECTION_ATTEMPTS + " attempts");
                    return;
                }
            }
            connectionAttempts++;
        }

        ServerPrinter s = new ServerPrinter(inFromServer);
        s.start();

        while (true) {
            String command = null;
            try {
                command = inFromUser.readLine();
            } catch (IOException e) {
                continue;
            }
            if (command.equals("quit")) {
                try {
                    clientSocket.close();
                    outToServer.close();
                } catch(IOException e) {
                    // don't care, we're ending anyway
                }
                break;
            } else {
                boolean sentToServer = false;
                int commandSendAttempts = 0;
                while (!sentToServer) {
                    try {
                        outToServer.writeBytes(command + '\n');
                        sentToServer = true;
                    } catch (IOException e) {
                        // keep trying to send the command
                        if(commandSendAttempts > MAX_COMMAND_SEND_ATTEMPTS) {
                            System.out.println("failed to send command '" + command + "' after "
                                    + MAX_COMMAND_SEND_ATTEMPTS + " attempts.");
                            break;
                        }
                    }
                    commandSendAttempts++;
                }
            }
        }

        s.exit();

        try {
            inFromServer.close();
        } catch(IOException e) {
            // don't care, we're exiting anyways
        }
    }

    private static InetAddress getIp(BufferedReader in) throws IOException {
        System.out.print("Enter the ip address you want to connect to: ");
        String ipAddrStr = in.readLine();
        InetAddress addr = null;
        boolean unknownHost = true;
        while (unknownHost) {
            try {
                addr = Inet4Address.getByName(ipAddrStr);
                unknownHost = false;
            } catch (UnknownHostException e) {
                System.out.println("That host doesn't exist! Try a different host.");
                System.out.print("Enter the ip address you want to connect to: ");
                ipAddrStr = in.readLine();
            }
        }
        return addr;
    }

    private static int getPort(BufferedReader in) throws IOException {
        int port = -1;
        System.out.print("Enter a port. Simply press <Enter> if you want the default port: ");
        String portStr = in.readLine();
        if(portStr.equals("")) {
            return DEFAULT_PORT;
        }
        boolean notParseable = true;
        while (notParseable) {
            try {
                port = Integer.parseInt(portStr);
                if (port > MAX_PORT || port < MIN_PORT) {
                    System.out.println("Your port should be between " + MAX_PORT + " and " + MIN_PORT);
                    throw new NumberFormatException();
                } else {
                    notParseable = false;
                }
            } catch (NumberFormatException e) {
                System.out.println("That port was invalid.");
                System.out.print("Enter a port: ");
                portStr = in.readLine();
            }
        }
        return port;
    }

    // this class prints everything it recieves from the server.
    private static class ServerPrinter extends Thread {
        private volatile boolean exit = false;
        BufferedReader inFromServer;

        public ServerPrinter (BufferedReader inFromServer) {
            this.inFromServer = inFromServer;
        }

        public void exit() {
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
