import java.io.*;
import java.net.*;
import java.util.*;

public class MudServer {
    // configuration files
    private static final String MOB_FILE = "mobs.txt";
    private static final String BLOCKS_FILE = "blocks.txt";
    private static final String ITEMS_FILE = "blocks.txt";

    // world save file
    private static final String WORLD_SAVE = "world-save.txt";

    private static int NUM_MOB_TYPES = 0;    
    private static World world;
    private static Value.ValueSet<Block> blocks;
    private static Value.ValueSet<Item> items;
    private static String[] allDrops;
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
        allDrops = new String[set.size()];
        int j = 0;
        for(String s : set) {
            allDrops[j] = s;
            j++;
        }

        blocks = Value.getValueFromFile(BLOCKS_FILE, new Block());
        items = Value.getValueFromFile(ITEMS_FILE, new Item());

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

    // calculates the actual move position given the origin, direction to move in, distance to attempt to travel,
    // the mob map, and the world map.
    private static int move(int xOrigin, int yOrigin, boolean xAxis, int numUnits) {
        if(numUnits == 0 && xAxis) {
            return xOrigin;
        } else if (numUnits == 0 && !xAxis) {
            return yOrigin;
        }

        if(xAxis) {
            int bounded = bound(xOrigin + numUnits, 0, World.MAP_SIZE - 1);
            for(int x = xOrigin + sign(numUnits); x != bounded; x += sign(numUnits)) {
                if((Boolean)world.getBlock(x, yOrigin).getStats().get("solid")) {
                    return x - sign(numUnits);
                } else if(world.hasMob(x, yOrigin)) {
                    Mob m = world.getMob(x, yOrigin, MOB_FILE);
                    if(RandUtils.rand(0, 99) < (Integer)m.getBaseStats().get("aggression")) {
                        System.out.println("you were blocked by a mob!");
                        return x;
                    }
                }
            }
            boolean solid = (Boolean)world.getBlock(bounded, yOrigin).getStats().get("solid");
            return solid ? bounded - sign(numUnits) : bounded;
        } else {
            int bounded = bound(yOrigin + numUnits, 0, World.MAP_SIZE - 1);
            for(int y = yOrigin + sign(numUnits); y != bounded; y += sign(numUnits)) {
                if((Boolean)world.getBlock(xOrigin, y).getStats().get("solid")) {
                    return y - sign(numUnits);
                } else if(world.hasMob(xOrigin, y)) {
                    Mob m = world.getMob(xOrigin, y, MOB_FILE);
                    if(RandUtils.rand(0, 99) < (Integer)m.getBaseStats().get("aggression")) {
                        System.out.println("you were blocked by a mob!");
                        return y;
                    }
                }
            }
            boolean solid = (Boolean)world.getBlock(xOrigin, bounded).getStats().get("solid");
            return solid ? bounded - sign(numUnits) : bounded;
        }
    }

    private static int sign(int a) {
        if(a > 0) {
            return 1;
        } else if (a == 0) {
            return 0;
        } else {
            return -1;
        }
    }

    private static StringBuilder map(int chunkSize) {
        StringBuilder s = new StringBuilder();
        for(int y = 0; y < World.MAP_SIZE; y += chunkSize) {
            s.append("|");
            for(int x = 0; x < World.MAP_SIZE; x += chunkSize) {
                boolean hasPlayer = false;
                for(Player player : accept.players()) {
                    if(player.x() >= x && player.x() < x + chunkSize && player.y() > y && player.y() <= y + chunkSize) {
                        s.append("\033[0m" + player.playerRep);
                        hasPlayer = true;
                        break;
                    }
                }
                if(!hasPlayer) {
                    int majorityBlock = getMajorityBlockInChunk(x, y, chunkSize);
                    int asciiColor = (Integer)blocks.get(majorityBlock).getStats().get("display");
                    if(asciiColor == -1) {
                        s.append("  ");
                    } else {
                        s.append("\033[48;5;" + asciiColor + "m  ");
                    }
                }
            }
            s.append("\033[0m|\n");
        }
        return s;
    }

    private static int getMajorityBlockInChunk(int xOrigin, int yOrigin, int chunkSize) {
        ArrayList<Integer> blockList = new ArrayList<Integer>();
        for(int x = xOrigin; x < xOrigin + chunkSize; x++) {
            for(int y = yOrigin; y < yOrigin + chunkSize; y++) {
                Block b = world.getBlock(x, y);
                while(blockList.size() <= b.getID()) {
                    blockList.add(0);
                }
                int mapWeight;
                if(b.getStats().hasVariable("map-weight")) {
                    mapWeight = (Integer)b.getStats().get("map-weight");
                } else {
                    mapWeight = 1;
                }
                blockList.set(b.getID(), blockList.get(b.getID()) + mapWeight);
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

    private static StringBuilder display(int dist, int xView, int yView) {
        StringBuilder s = new StringBuilder();
        for(int y = max(0,yView - dist); y < min(World.MAP_SIZE, yView + dist + 1); y++) {
            s.append("|");
            for(int x = max(0,xView - dist); x < min(World.MAP_SIZE, xView + dist + 1); x++) {
                boolean hasPlayer = false;
                for(Player player : accept.players()) {
                    if(player.x() == x && player.y() == y) {
                        s.append("\033[0m" + player.playerRep);
                        hasPlayer = true;
                        break;
                    }
                }
                if(!hasPlayer) {
                    Block b = world.getBlock(x, y);
                    int asciiColor = (Integer)b.getStats().get("display");
                    boolean hideMob = b.getStats().hasVariable("hide-mobs") && (Boolean)b.getStats().get("hide-mobs");
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
        return s;
    }

    private static StringBuilder playerGive(String command, Player player) {
        StringBuilder out = new StringBuilder("");
        Scanner scan = new Scanner(command);
        scan.next();
        int id = scan.nextInt();
        String item = scan.next();
        int amount = scan.nextInt();
        scan.close();
        if(id < 0 || id >= accept.players().size()) {
            out.append("That player doesn't exist!");
            return out;
        }
        Player recipient = accept.players().get(id);
        if(!player.getInventory().hasVariable(item)) {
            out.append("You don't have that item!");
            return out;
        }
        if(amount < 0) {
            out.append("Nice try, but you can't give negative donations.");
            return out;
        }
        if((Integer)player.getInventory().get(item) < amount) {
            out.append("You have ");
            out.append((Integer)player.getInventory().get(item) + "");
            out.append(" of that item, not ");
            out.append(amount + ".");
            return out;
        }
        recipient.addToInventory(item, amount);
        player.removeFromInventory(item, amount);
        return out;
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

    private static void respawn(Player player) {
        player.clearInventory();
        player.resetToBaseStats();
        player.mob = null;
        
        int spawnX = RandUtils.rand(0, World.MAP_SIZE - 1);
        int spawnY = RandUtils.rand(0, World.MAP_SIZE - 1);
        Block b = world.getBlock(spawnX, spawnY);
        while((Boolean)b.getStats().get("solid") || ((String)b.getStats().get("name")).contains("water") || world.hasMob(spawnX, spawnY)) {
            spawnX = RandUtils.rand(0, World.MAP_SIZE - 1);
            spawnY = RandUtils.rand(0, World.MAP_SIZE - 1);
            b = world.getBlock(spawnX, spawnY);
        }
    }

    private static StringBuilder mobAttack(Mob mob, Player player) {
        StringBuilder out = new StringBuilder("");
        out.append(mob.getBaseStats().get("name") + ": " + mob.getQuote("attack") + "\n");
        out.append(mob.getBaseStats().get("name") + " attacked you and dealt " + mob.getBaseStats().get("dmg") + " damage.\n");
        player.changeStat("health", -(Integer)mob.getStats().get("dmg"));
        if(player.isDead()) {
            out.append(mob.getBaseStats().get("name") + ": " + mob.getQuote("mob-victory") + "\n");
            out.append("You were killed by " + mob.getBaseStats().get("name") + "\n");
            respawn(player);
            out.append("Respawning at " + player.x() + ", " + player.y() + "\n");
        }
        return out;
    }

    private static StringBuilder playerAttack(Mob mob, Player player) {
        StringBuilder out = new StringBuilder("");
        out.append("You attacked " + mob.getBaseStats().get("name") + " and dealt " + player.getBaseStats().get("dmg") + " damage.\n");
        mob.changeStat("health", -(Integer)player.getBaseStats().get("dmg"));
        if(mob.isDead()) {
            out.append(mob.getBaseStats().get("name") + ": " + mob.getQuote("player-victory") + "\n");
            out.append("You murdered " + mob.getBaseStats().get("name") + "\n");

            out.append("You got " + mob.getBaseStats().get("xp") + " xp.\n");
            player.changeStat("xp", (Integer)mob.getBaseStats().get("xp"));

            String[] drops = mob.getDrops();
            for(String drop : drops) {
                out.append("You got " + drop + "\n");
                player.addToInventory(drop, 1);
            }
            player.mob = null;
        }
        return out;
    }

    public static StringBuilder handleCommand(String command, Player player) {
        StringBuilder out = new StringBuilder("");
        if(command.equals("")) {
            command = player.lastCommand;
        } else {
            player.lastCommand = command;
        }

        if(command.startsWith("eat")) {
            Scanner scan = new Scanner(command);
            scan.next();
            String item = scan.next();
            int amount = scan.nextInt();
            scan.close();
            if(!player.getInventory().hasVariable(item)) {
                out.append("You don't have that item!");
                return out;
            }
            if((Integer)player.getInventory().get(item) < amount) {
                out.append("You have only " + ((Integer)player.getInventory().get(item)) + " of that item.");
                return out;
            }
            int healthGain = 0;
            try {
                healthGain = (Integer)items.get(item).getStats().get("health-gain");
            } catch (IllegalArgumentException e) {
                out.append("You can't eat that!");
                return out;
            }
            player.removeFromInventory(item, amount);
            player.changeStat("health", amount * healthGain);
            out.append("You got " + (amount * healthGain) + " health.");
            return out;
        }

        if(player.mob != null) {
            if(command.equals("attack")) {
                out.append(playerAttack(player.mob, player));
                if(player.mob != null) {
                    out.append(mobAttack(player.mob, player));
                }
            } else if(command.equals("trade")) {
                try {
                    int numItems = (Integer)player.mob.getBaseStats().get("trade");
                    int xp = (Integer)player.mob.getBaseStats().get("trade-xp");
                    Random XYRand = world.getXYRand(player.x(), player.y());
                    out.append("I can trade " + xp + " xp for each: \n");
                    String[] trades = new String[numItems];
                    for(int i = 0; i < trades.length; i++) {
                        trades[i] = allDrops[XYRand.nextInt(allDrops.length)];
                        out.append((i + 1) + ". " + trades[i].replace(' ', '-') + "\n");
                    }
                    out.append("use 'trade <item#> <amount>' to trade.\n");
                } catch(IllegalArgumentException e) {
                    out.append("You can't trade with " + (String)player.mob.getStats().get("name") + "!");
                }
            } else if(command.startsWith("trade")) {
                try {
                    int numItems = (Integer)player.mob.getBaseStats().get("trade");
                    int xp = (Integer)player.mob.getBaseStats().get("trade-xp");
                    Random XYRand = world.getXYRand(player.x(), player.y());
                    String[] trades = new String[numItems];
                    for(int i = 0; i < trades.length; i++) {
                        trades[i] = allDrops[XYRand.nextInt(allDrops.length)];
                    }
                    Scanner scan = new Scanner(command);
                    scan.next();
                    int itemNumber = scan.nextInt();
                    int numToTrade = scan.nextInt();
                    scan.close();
                    String drop = trades[itemNumber].replace(' ', '-');
                    if(!player.getInventory().hasVariable(drop)) {
                        out.append("You don't have that item!");
                        return out;
                    }
                    if((Integer)player.getInventory().get(drop) < numToTrade) {
                        out.append("You have only " + ((Integer)player.getInventory().get(drop)) + " of that item.");
                        return out;
                    }
                    player.removeFromInventory(drop, numToTrade);
                    player.changeStat("xp", xp * numToTrade);
                } catch(IllegalArgumentException e) {
                    out.append("You can't trade with " + (String)player.mob.getStats().get("name") + "!");
                }
            } else if(command.equals("run")) {

            }
            return out;
        }

        if(command.equals("map")) {
            out.append(map(30));
        } else if(command.equals("disp")) {
            out.append(display((Integer)player.getBaseStats().get("view"), player.x(), player.y()));
        } else if(command.startsWith("give")) {
            out.append(playerGive(command, player));
        } else if(command.charAt(0) == 'w' || command.charAt(0) == 'a' || command.charAt(0) == 's' || command.charAt(0) == 'd') { // movement
            int dist;
            if(command.length() > 1) {
                try {
                    dist = Integer.parseInt(command.substring(1, command.length()));
                    if(dist > (Integer)player.getStats().get("speed")) {
                        out.append("You can't move that far! Upgrade your speed stat to go farther each turn.");
                        return out;
                    }
                } catch (NumberFormatException e) {
                    out.append("You didn't type a number after the movement character.");
                    return out;
                }
            } else {
                dist = (Integer)player.getStats().get("speed");
            }
            if(command.charAt(0) == 'w') {
                int actualPosn = move(player.x(), player.y(), false, -dist);
                player.moveTo(player.x(), actualPosn);
            } else if (command.charAt(0) == 'a') {
                int actualPosn = move(player.x(), player.y(), true, -dist);
                player.moveTo(actualPosn, player.y());
            } else if (command.charAt(0) == 's') {
                int actualPosn = move(player.x(), player.y(), false, dist);
                player.moveTo(player.x(), actualPosn);
            } else if (command.charAt(0) == 'd') {
                int actualPosn = move(player.x(), player.y(), true, dist);
                player.moveTo(actualPosn, player.y());
            }

            if(!world.hasMob(player.x(), player.y())) {
                out.append(display((Integer)player.getBaseStats().get("view"), player.x(), player.y()));
            } else {
                Mob mob = world.getMob(player.x(), player.y(), MOB_FILE);

                // mob entrance
                out.append("You encountered " + mob.getBaseStats().get("name") + "\n");
                out.append(mob.getBaseStats().get("name") + ": " + mob.getQuote("entrance"));
                out.append(mob.getImg());

                player.mob = mob;

                if((Integer)mob.getStats().get("speed") > (Integer)player.getStats().get("speed")) {
                    mobAttack(mob, player);
                }
            }
        }
        return out;
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

class PlayerThread extends Thread {
    private BufferedReader inFromClient;
    private DataOutputStream outToClient;
    public final Player player;
    private boolean continueRun = true;
    public PlayerThread(Socket conn, int ID) throws IOException {
        inFromClient = new BufferedReader(new InputStreamReader(conn.getInputStream()));
        outToClient = new DataOutputStream(conn.getOutputStream());
        player = new Player(500, 500);
        player.playerRep = ID + "" + ID;
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
                        outToClient.writeUTF(line + "\n");
                        System.out.println(line);
                    }
                } 
                // if MudServer.handleCommand breaks in some way, print the error, but don't crash the players session.
                // also, notify the player that the action they tried to do didn't work.
                catch(Exception e) {
                    outToClient.writeUTF("That action didn't work. Check your syntax.\n/end/\n");
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