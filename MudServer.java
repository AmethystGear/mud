import java.io.*;
import java.net.*;
import java.util.*;

public class MudServer {
    // configuration files
    private static final String MOB_FILE = "mobs.txt";
    private static final String BLOCKS_FILE = "blocks.txt";

    // world save file
    private static final String WORLD_SAVE = "world-save.txt";

    private static int NUM_MOB_TYPES = 0;
    
    private static World world;
    private static Block.BlockSet blocks;
    private static Accept accept;

    public static void main(String[] args) {
        // find all drops, and find the number of mobs.
        Set<String> set = new HashSet<String>();
        Scanner fr;
        try {
            fr = new Scanner(new File(MOB_FILE));
        } catch(FileNotFoundException e) {
            System.out.println("Can't find mobs.txt!");
            return;
        }

        while (fr.hasNextLine()) {
            String line = fr.nextLine();
            if(line.trim().equals("/begin/")) {
                NUM_MOB_TYPES++;
            } else {
                Scanner tok = new Scanner(line);
                if(tok.hasNext()) {
                    tok.next();
                }
                if(tok.hasNext() && tok.next().equals("drops")) {
                    String[] drops = ScannerUtils.getRemainingInputAsStringArray(tok);
                    for(String drop : drops) {
                        set.add(drop);
                    }
                }
            }
        }
        String [] allDrops = new String[set.size()];
        int j = 0;
        for(String s : set) {
            allDrops[j] = s;
            j++;
        }

        blocks = Block.getBlocksFromFile(BLOCKS_FILE);

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
            world = new World(RandUtils.rand(0, Integer.MAX_VALUE - 1), NUM_MOB_TYPES, blocks);
        } else {
            try {
                world = new World(WORLD_SAVE, NUM_MOB_TYPES, blocks);
            } catch(FileNotFoundException e) {
                System.out.println("There is no saved world! Creating new world instead.");
                world = new World(RandUtils.rand(0, Integer.MAX_VALUE - 1), NUM_MOB_TYPES, blocks);
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
                    throw new NumberFormatException();
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
            if(in.nextLine().equals("quit")) {
                quit = true;
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

    private static String map(int chunkSize) {
        StringBuilder s = new StringBuilder();
        for(int y = 0; y < World.MAP_SIZE; y += chunkSize) {
            s.append("|");
            for(int x = 0; x < World.MAP_SIZE; x += chunkSize) {
                boolean hasPlayer = false;
                for(Player player : accept.players()) {
                    if(player.x() >= x && player.x() < x + chunkSize && player.y() > y && player.y() <= y + chunkSize) {
                        s.append(player.playerRep);
                        hasPlayer = true;
                        break;
                    }
                }
                if(!hasPlayer) {
                    int majorityBlock = getMajorityBlockInChunk(x, y, chunkSize);
                    int asciiColor = (Integer)blocks.getBlock(majorityBlock).STATS.get("display");
                    if(asciiColor == -1) {
                        s.append("  ");
                    } else {
                        s.append("\033[48;5;" + asciiColor + "m__\033[0m");
                    }
                }
            }
            s.append("|\n");
        }
        return s.toString();
    }

    private static int getMajorityBlockInChunk(int xOrigin, int yOrigin, int chunkSize) {
        ArrayList<Integer> blockList = new ArrayList<Integer>();
        for(int x = xOrigin; x < xOrigin + chunkSize; x++) {
            for(int y = yOrigin; y < yOrigin + chunkSize; y++) {
                Block b = world.getBlock(x, y);
                while(blockList.size() <= b.BLOCK_ID) {
                    blockList.add(0);
                }
                int mapWeight;
                if(b.STATS.hasVariable("map-weight")) {
                    mapWeight = (Integer)b.STATS.get("map-weight");
                } else {
                    mapWeight = 1;
                }
                blockList.set(b.BLOCK_ID, blockList.get(b.BLOCK_ID) + mapWeight);
            }
        }
        int maxIndex = 0;
        for(int i = 1; i < blockList.size(); i++) {
            if(blockList.get(i) > blockList.get(maxIndex)) {
                maxIndex = i;
            }
        }
        return maxIndex;
    }

    private static String display(int dist, int xView, int yView) {
        StringBuilder s = new StringBuilder();
        for(int y = max(0,yView - dist); y < min(World.MAP_SIZE, yView + dist + 1); y++) {
            s.append("\033[0m|");
            for(int x = max(0,xView - dist); x < min(World.MAP_SIZE, xView + dist + 1); x++) {
                boolean hasPlayer = false;
                for(Player player : accept.players()) {
                    if(player.x() == x && player.y() == y) {
                        s.append(player.playerRep);
                        hasPlayer = true;
                        break;
                    }
                }
                if(!hasPlayer) {
                    Block b = world.getBlock(x, y);
                    int asciiColor = (Integer)b.STATS.get("display");
                    boolean hideMob = b.STATS.hasVariable("hide-mobs") && (Boolean)b.STATS.get("hide-mobs");
                    if(asciiColor == -1) {
                        if(!hideMob && world.hasMob(x, y)) {
                            s.append("??");
                        } else {
                            s.append("  ");
                        }
                    } else {
                        if(!hideMob && world.hasMob(x, y)) {
                            s.append("\033[48;5;" + asciiColor + "m??");
                        } else {
                            s.append("\033[48;5;" + asciiColor + "m  ");
                        }
                        
                    }
                }
            }
            s.append("\033[0m|\n");
        }
        return s.toString();
    }

    private static int min(int a, int b) {
        return a < b ? a : b;
    }

    private static int max(int a, int b) {
        return a > b ? a : b;
    }

    private static int bound(int a, int min, int max) {
        return max(min, min(max, a));
    }

    public static String handleCommand(String command, Player player) {
        StringBuilder out = new StringBuilder("");
        if(command.equals("map")) {
            out.append(map(30));
        } else if(command.equals("disp")) {
            out.append(display(10, player.x(), player.y()));
        }
        out.append("\n/end/\n");
        return out.toString();
    }
}

class Accept extends Thread {
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
                players.add(p);
                ID++;
                p.run();
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

class PlayerThread extends Thread {
    private BufferedReader inFromClient;
    private DataOutputStream outToClient;
    public final Player player;
    private boolean continueRun = true;
    public PlayerThread(Socket conn, int ID) throws IOException {
        inFromClient = new BufferedReader(new InputStreamReader(conn.getInputStream()));
        outToClient = new DataOutputStream(conn.getOutputStream());
        player = new Player(0, 0);
        player.playerRep = ID + "" + ID;
    }

    public void run() {
        while(continueRun) {
            try {
                String command = inFromClient.readLine();
                System.out.println(command);
                String output = MudServer.handleCommand(command, player);
                System.out.println(output);
                outToClient.writeUTF(output);
            } catch(IOException e) {
                System.out.println("command failed!");
            }
        }
    }

    public void end() {
        continueRun = false;
    }
}