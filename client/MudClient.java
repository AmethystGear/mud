package client;

import java.io.*;
import java.net.*;

public class MudClient {
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

        while (true) {
            System.out.print("Enter a command: ");
            String command = inFromUser.readLine();
            if (command.equals("quit")) {
                clientSocket.close();
            } else {
                outToServer.writeBytes(command + '\n');
                String line = inFromServer.readLine();
                while (!line.contains("/end/")) {
                    int index = line.indexOf("/begin/");
                    if (index != -1) {
                        System.out.println(line.substring(index + 7, line.length()));
                    }
                    line = inFromServer.readLine();
                }
            }
        }
    }
}