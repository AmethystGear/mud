package server.main;

import server.actions.*;
import server.objects.Player;
import server.utils.RandUtils;

import java.io.*;
import java.net.*;
import java.util.*;

public class MudServer {
    // world save file
    private static final String WORLD_SAVE = "save/world-save.txt";
    // player save file
    private static final String PLAYER_SAVE = "save/player-save.txt";
  
    private static World world;
    private static Accept accept;

    public static StringBuilder handleCommand(String command, Player player) {
        List<Player.ReadOnlyPlayer> players = new ArrayList<>();
        for (Player p : accept.players()) {
            players.add(new Player.ReadOnlyPlayer(p));
        }

        if(command.equals("") && player.lastAction != null) {
            StringBuilder error = new StringBuilder("");
            Action newAction;
            try {
                // create new instance of whatever action that we have.
                newAction = player.lastAction.getClass().getConstructor().newInstance();
            } catch (Exception e) {
                // we should never be in this state. If we are, there's a bug.
                throw new RuntimeException("couldn't create new instance of player's last action!");
            }
            if(newAction.parseCommand(player.lastCommand, new Player.ReadOnlyPlayer(player), players, world, error)) {
                return newAction.run(player, accept.players(), world);
            } else {
                error.append("\n");
                return error;
            }
        } else if (command.equals("")) {
            return new StringBuilder("You have no commands in your history!");
        }

        System.out.println(command);
        for (Action a : Actions.actions) {
            if (a.matchCommand(command)) {
                try {
                    StringBuilder error = new StringBuilder("");
                    if (a.parseCommand(command, new Player.ReadOnlyPlayer(player), players, world, error)) {
                        player.lastCommand = command;
                        player.lastAction = a;
                        return a.run(player, accept.players(), world);
                    } else {
                        error.append("\n");
                        return error;
                    }
                } catch (Exception e) {
                    // We should never be in this state. If we are, there's a bug.
                    System.out.println("action parse failed.");
                    System.out.println(e);
                    e.printStackTrace();
                }
            }
        }
        return new StringBuilder("no action matches your input!");
    }

    public static void main(String[] args) throws FileNotFoundException {
        // load all the player accounts.
        Accounts.load(new Scanner(new File(PLAYER_SAVE)));

        boolean makeNewWorld;
        Scanner in = new Scanner(System.in);
        System.out.print("Do you want to load your saved world, or create a new one?(load/create): ");
        String inp = in.nextLine();
        while(!inp.equals("load") && !inp.equals("create")) {
            System.out.print("Please type load or create: ");
            inp = in.nextLine();
        }
        makeNewWorld = inp.equals("create");

        if(makeNewWorld) {
            world = new World(RandUtils.rand(0, Integer.MAX_VALUE - 1));
        } else {
            try {
                world = new World(WORLD_SAVE);
            } catch(FileNotFoundException e) {
                System.out.println("There is no saved world! Creating new world instead.");
                world = new World(RandUtils.rand(0, Integer.MAX_VALUE - 1));
            }
        }

        int port = -1;
        System.out.print("Enter a port: ");
        String portStr = in.nextLine();
        boolean notParseable = true;
        while(notParseable) {
            try {
                port = Integer.parseInt(portStr);
                if(port > 65535 || port < 0) {                    
                    System.out.println("That port was invalid.");
                    System.out.print("Enter a port: ");
                    portStr = in.nextLine();
                } else {
                    notParseable = false;
                }
            } catch(NumberFormatException e) {
                System.out.println("That port was invalid.");
                System.out.print("Enter a port: ");
                portStr = in.nextLine();
            }
        }
        ServerSocket server = null;
        try {
            server = new ServerSocket(port);
        } catch (IOException e) {
            System.out.println("could not open server");
            in.close();
            return;
        }

        accept = new Accept(server);
        accept.start();

        boolean quit = false;
        while(!quit) {
            String line = in.nextLine();
            if(line.equals("quit")) {
                quit = true;
            }
            if(line.equals("save")) {
                world.saveTo(WORLD_SAVE);
                Accounts.save(new PrintWriter(new File(PLAYER_SAVE)));
            }
        }

        accept.end();
        try {
            server.close();
        } catch(IOException e) {
            // don't care, we're exiting anyways.
        }
        in.close();
    }

    private static class Accept extends Thread {
        private ServerSocket server;
        private ArrayList<PlayerThread> players;
        int ID = 0;
        private boolean continueRun = true;
        public Accept(ServerSocket server) {
            this.server = server;
            this.players = new ArrayList<PlayerThread>();
        }
    
        public void run() {
            while(continueRun) {
                try {
                    PlayerThread p = new PlayerThread(server.accept(), ID);
                    System.out.println("connected!");
                    players.add(p);
                    ID++;
                    p.start();
                } catch (IOException e) {
                    System.out.println("client connection failed!");
                }
            }
        }
    
        public void end() {
            for(PlayerThread t : players) {
                t.end();
            }
            continueRun = false;
        }
    
        public List<Player> players() {
            List<Player> p = new ArrayList<Player>();
            for(PlayerThread t : players) {
                p.add(t.player);
            }
            return p;
        }
    }
    
    private static class PlayerThread extends Thread {
        private BufferedReader inFromClient;
        private DataOutputStream outToClient;
        public final Player player;
        private boolean continueRun = true;
        public PlayerThread(Socket conn, int ID) throws IOException {
            inFromClient = new BufferedReader(new InputStreamReader(conn.getInputStream()));
            outToClient = new DataOutputStream(conn.getOutputStream());
            player = new Player(0, 0);
            player.playerRep = ID + "" + ID;
            player.respawn(MudServer.world);
        }
    
        public void run() {
            while(continueRun) {
                try {
                    String command = inFromClient.readLine();
                    try {
                        StringBuilder output = MudServer.handleCommand(command, player);             
                        output.append("\n/end/\n");                                   
                        Scanner scan = new Scanner(output.toString());
                        while(scan.hasNextLine()) {
                            String line = scan.nextLine();
                            outToClient.writeUTF("/begin/" + line + "\n");
                        }
                    } 
                    // if MudServer.handleCommand breaks in some way, print the error, but don't crash the players session.
                    // also, notify the player that the action they tried to do didn't work.
                    catch(Exception e) {
                        outToClient.writeUTF("That action didn't work. \n/end/\n");
                        System.out.println(e);
                        e.printStackTrace();
                    }
                } catch(IOException e) {
                    //ignore errors
                }
            }
        }
    
        public void end() {
            continueRun = false;
        }
    }
}

