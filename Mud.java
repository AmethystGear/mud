import java.io.*;
import java.util.*;

public class Mud {
    private static final int MAP_SIZE = 3000;
    private static final int MOB_SPAWN_CHANCE = 1;
    private static final int MOB_SPAWN_CHANCE_GRASS = 10;
    private static final int GRASS_BLOCK_SPAWN_CHANCE = 10;
    private static final int GRASS_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR = 65;
    private static final int FILLED_BLOCK_SPAWN_CHANCE = 5;
    private static final int FILLED_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR = 50;

    private static final String MOB_FILE = "mobs.txt";
    private static final String BLOCKS_FILE = "blocks.txt";
    private static final String STATS_SAVE = "stats-save.txt";
    private static final String INVENTORY_SAVE = "inventory-save.txt";

    private static int NUM_MOB_TYPES = 0;

    private static boolean hasNeighbor(int x, int y, int type, int[][] worldMap) {
        return worldMap[min(MAP_SIZE - 1, x + 1)][y] == type || worldMap[x][min(MAP_SIZE - 1, y + 1)] == type
                || worldMap[x][max(0, y - 1)] == type || worldMap[max(0, x - 1)][y] == type;
    }

    private static boolean fileExists(String file) {
        return new File(file).exists() && !new File(file).isDirectory();
    }


    private static void spawnVillage(int xOrigin, int yOrigin, int [][] worldMap, int[][] mobMap, Block.BlockSet blocks) {
        int floor = blocks.getBlock("village floor").BLOCK_ID;
        int villageLength = RandUtils.rand(4, 15) * 5;
        int pathSize = RandUtils.rand(3, 5);
        for(int x = xOrigin; x < xOrigin + villageLength; x++) {
            for(int y = yOrigin; y < yOrigin + pathSize; y++) {
                worldMap[x][y] = floor;
                mobMap[x][y] = 0;
            }
        }
        boolean generateUp = false;
        for(int x = xOrigin + RandUtils.rand(2, 5); x < xOrigin + villageLength; x+= RandUtils.rand(5, 10)) {
            int pathlen = RandUtils.rand(3, 10);
            int hutSize = RandUtils.rand(2, 4);
            if(generateUp) {
                for(int y = yOrigin; y > yOrigin - pathlen; y--) {
                    worldMap[x][y] = floor;
                    mobMap[x][y] = 0;
                }
                spawnHut(x - hutSize, yOrigin - pathlen - hutSize * 2 + 1, hutSize, worldMap, mobMap, blocks);
            } else {
                for(int y = yOrigin + pathSize; y < yOrigin + pathlen + pathSize; y++) {
                    worldMap[x][y] = floor;
                    mobMap[x][y] = 0;
                }
                spawnHut(x - hutSize, yOrigin + pathlen + pathSize, hutSize, worldMap, mobMap, blocks);
            }
            generateUp = !generateUp;
        }
    }

    private static void spawnHut(int xOrigin, int yOrigin, int size, int [][] worldMap, int[][] mobMap, Block.BlockSet blocks) {
        int floor = blocks.getBlock("village floor").BLOCK_ID;
        int wall = blocks.getBlock("village wall").BLOCK_ID;
        int surveyor = blocks.getBlock("surveyor").BLOCK_ID;
        int surveyorSpawnChance = (Integer)blocks.getBlock("surveyor").STATS.get("spawn-chance");
        size = size * 2 + 1;
        for(int x = xOrigin; x < xOrigin + size; x++) {
            for(int y = yOrigin; y < yOrigin + size; y++) {
                worldMap[x][y] = floor;
                mobMap[x][y] = 0;
                if(RandUtils.rand(0, 99) < surveyorSpawnChance) {
                    worldMap[x][y] = surveyor;
                }
            }
        }
        for(int x = xOrigin; x < xOrigin + size; x++) {
            if(x - xOrigin != size/2) {
                worldMap[x][yOrigin] = wall;
                worldMap[x][yOrigin + size - 1] = wall;
            }
        }
        for(int y = yOrigin; y < yOrigin + size; y++) {
            worldMap[xOrigin][y] = wall;
            worldMap[xOrigin + size - 1][y] = wall;
        }
    }

    public static void main(String[] args) throws Exception {

        // find all drops, and find the number of mobs.
        Set<String> set = new HashSet<String>();
        Scanner fr = new Scanner(new File(MOB_FILE));
        while (fr.hasNextLine()) {
            String line = fr.nextLine();
            if(line.trim().equals("/begin/")) {
                NUM_MOB_TYPES++;
            } else {
                Scanner tok = new Scanner(line);
                if(tok.hasNext() && tok.next().equals("drops:")) {
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

        Block.BlockSet blocks = Block.getBlocksFromFile(BLOCKS_FILE);
        
        int[][] worldMap = new int[MAP_SIZE][];
        for(int i = 0; i < MAP_SIZE; i++) {
            worldMap[i] = new int[MAP_SIZE];
        }

        int[][] mobMap = new int[MAP_SIZE][];
        for(int i = 0; i < MAP_SIZE; i++) {
            mobMap[i] = new int[MAP_SIZE];
        }

        // create map
        for(int x = 0; x < MAP_SIZE; x++) {
            for(int y = 0; y < MAP_SIZE; y++) {
                int empty = blocks.getBlock("empty").BLOCK_ID;
                int grass = blocks.getBlock("grass").BLOCK_ID;
                int rock = blocks.getBlock("rock").BLOCK_ID;

                int blockType = empty;
                if(RandUtils.rand(0, 99) < GRASS_BLOCK_SPAWN_CHANCE) {
                    blockType = grass;
                }
                if(hasNeighbor(x, y, grass, worldMap) && RandUtils.rand(0, 99) < GRASS_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR) {
                    blockType = grass;
                }
                if(RandUtils.rand(0, 99) < FILLED_BLOCK_SPAWN_CHANCE) {
                    blockType = rock;
                }
                if(hasNeighbor(x, y, rock, worldMap) && RandUtils.rand(0, 99) < FILLED_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR) {
                    blockType = rock;
                }
                worldMap[x][y] = blockType;

                if(blockType == 0) {
                    if(RandUtils.rand(0, 99) < MOB_SPAWN_CHANCE) {
                        mobMap[x][y] = RandUtils.rand(1, NUM_MOB_TYPES);
                    }
                } else if(blockType == 1) {
                    if(RandUtils.rand(0, 99) < MOB_SPAWN_CHANCE_GRASS) {
                        mobMap[x][y] = RandUtils.rand(1, NUM_MOB_TYPES);
                    }
                }
            }
        }

        int numVillages = RandUtils.rand(300, 400);
        for(int i = 0; i < numVillages; i++) {
            int x = RandUtils.rand(500, 2500);
            int y = RandUtils.rand(500, 2500);
            System.out.println(x + ", " + y);
            spawnVillage(x, y, worldMap, mobMap, blocks);
        }
        
        // assign spawn location to a place that is open and doesn't have a mob.
        int spawnX = RandUtils.rand(0, MAP_SIZE - 1);
        int spawnY = RandUtils.rand(0, MAP_SIZE - 1);
        while(worldMap[spawnX][spawnY] == 2|| mobMap[spawnX][spawnY] != 0) {
            spawnX = RandUtils.rand(0, MAP_SIZE - 1);
            spawnY = RandUtils.rand(0, MAP_SIZE - 1);
        }

        Player player;

        if(fileExists(STATS_SAVE) && fileExists(INVENTORY_SAVE)) {
            player = new Player(spawnX, spawnY, STATS_SAVE, INVENTORY_SAVE);
        } else {
            player = new Player(spawnX, spawnY);
        }
        Scanner in = new Scanner(System.in);

        Mob mobToFight = null;
        boolean isFightingMob = false;
        String lastAction = "";
        //game loop
        while(true) {
            System.out.print("Enter a command: ");
            String action = in.nextLine();
            if(action.length() == 0) {
                action = lastAction;
            }
            lastAction = action;

            if(action.equals("quit")) {
                break;
            }
            if(action.equals("map")) {
                System.out.print(map(worldMap, blocks, 30, player));
                continue;
            }
            if(action.equals("tp")) {
                isFightingMob = false;
                System.out.print("Enter x: ");
                int x = Integer.parseInt(in.nextLine());
                System.out.print("Enter y: ");
                int y = Integer.parseInt(in.nextLine());
                player.moveTo(x, y);
            }
            if(action.equals("save")) {
                player.updateXP();
                player.getBaseStats().saveTo(STATS_SAVE);
                player.getInventory().saveTo(INVENTORY_SAVE);
                continue;
            }
            if(action.equals("stat")){
                System.out.println("Base stats: ");
                System.out.print(player.getBaseStats().toString());
                System.out.println("Stats: ");
                System.out.print(player.getStats().toString());
                continue;
            }
            if(action.equals("inv")){
                System.out.println("Inventory: ");
                System.out.print(player.getInventory().toString());
            }
            if(action.equals("mobstat")) {
                if(!isFightingMob) {
                    System.out.println("you are not currently fighting a mob!");
                } else {
                    System.out.println("Base stats: ");
                    System.out.print(mobToFight.getBaseStats().toString());
                    System.out.println("Stats: ");
                    System.out.print(mobToFight.getStats().toString());
                }
                continue;
            }
            if(action.equals("upgrade")) {
                System.out.print("Enter stat to upgrade: ");
                String stat = in.nextLine();
                try {
                    player.upgradeBaseStat(stat);
                } catch (IllegalArgumentException e) {
                    System.out.println("that stat doesn't exist!");
                }
                continue;             
            }
            if(isFightingMob) { // mob fight world
                if(action.equals("attack")) {
                    System.out.println("You attacked " + mobToFight.getBaseStats().get("name") + " and dealt " + player.getBaseStats().get("dmg") + " damage.");
                    mobToFight.changeStat("health", -(Integer)player.getBaseStats().get("dmg"));
                    if(mobToFight.isDead()) {
                        System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("player-victory"));
                        System.out.println("You murdered " + mobToFight.getBaseStats().get("name"));
                        mobMap[player.x()][player.y()] = 0; // remove mob from map
                        System.out.println("You got " + mobToFight.getBaseStats().get("xp") + " xp.");
                        player.changeStat("xp", (Integer)mobToFight.getBaseStats().get("xp"));

                        String[] drops = mobToFight.getDrops();
                        for(String drop : drops) {
                            System.out.println("You got " + drop);
                            player.addToInventory(drop);
                        }

                        isFightingMob = false;
                    } else {
                        System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("attack"));
                        System.out.println(mobToFight.getBaseStats().get("name") + " attacked you and dealt " + mobToFight.getBaseStats().get("dmg") + " damage.");
                        player.changeStat("health", -(Integer)mobToFight.getBaseStats().get("dmg"));
                        if(player.isDead()) {
                            System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("mob-victory"));
                            System.out.println("You were killed by " + mobToFight.getBaseStats().get("name"));
                            return;
                        }
                    }
                } else if(action.equals("run")) {
                    System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("player-run"));
                    System.out.println("You ran away from " + mobToFight.getBaseStats().get("name") + ".");
                    isFightingMob = false;
                } else if(action.equals("trade")) {
                    int numItems;
                    int xp;
                    try {
                        numItems = (Integer)mobToFight.getBaseStats().get("trade");
                        xp = (Integer)mobToFight.getBaseStats().get("trade-xp");
                    } catch(IllegalArgumentException e) {
                        System.out.println("You can't trade with " + mobToFight + "!");
                        numItems = -1;
                        xp = -1;
                    }
                    if(numItems != -1) {        
                        Random XYRand = RandUtils.getXYRand(player.x(), player.y());
                        System.out.println("I can trade " + xp + " xp for each: ");
                        String[] trades = new String[numItems];
                        for(int i = 0; i < trades.length; i++) {
                            trades[i] = allDrops[XYRand.nextInt(allDrops.length)];
                            System.out.println((i + 1) + ". " + trades[i]);
                        }
                        System.out.print("Enter which # item you wish to trade: ");
                        int itemNum = Integer.parseInt(in.nextLine()) - 1;
                        try {
                            int amount = (Integer)player.getInventory().get(trades[itemNum]);
                            System.out.print("You have " + amount + " of that item. How many do you wish to trade? ");
                            int numToTrade = Integer.parseInt(in.nextLine());
                            try {
                                player.removeFromInventory(trades[itemNum], numToTrade);
                                player.changeStat("xp", xp * numToTrade);
                            } catch(IllegalArgumentException e) {
                                System.out.println("You don't have enough of that item!");
                            }
                        } catch(IllegalArgumentException e) {
                            System.out.println("You don't have that item!");
                        }
                    }
                }
            } else { // actual world
                if(action.equals("disp")) { //display
                    if(blocks.getBlock("surveyor").BLOCK_ID == worldMap[player.x()][player.y()]) {
                        System.out.print("Enter how far: ");
                        int dist = Integer.parseInt(in.nextLine());
                        System.out.print("Enter which direction: ");
                        String dir = in.nextLine();
                        int xPos = player.x();
                        int yPos = player.y();
                        if(dir.contains("w")) {
                            yPos -= dist;
                        }
                        if(dir.contains("a")) {
                            xPos -= dist;
                        }
                        if(dir.contains("s")) {
                            yPos += dist;
                        }
                        if(dir.contains("d")) {
                            xPos += dist;
                        }
                        System.out.println(display(dist, xPos, yPos, worldMap, blocks));
                    } else {
                        System.out.println(display((Integer)player.getBaseStats().get("view"), player.x(), player.y(), worldMap, blocks));
                    }
                } else if(action.charAt(0) == 'w' || action.charAt(0) == 'a' || action.charAt(0) == 's' || action.charAt(0) == 'd') { // movement
                    int dist;
                    if(action.length() > 1) {
                        try {
                            dist = Integer.parseInt(action.substring(1, action.length()));
                            if(dist > (Integer)player.getStats().get("speed")) {
                                System.out.println("You can't move that far! Upgrade your speed stat to go farther each turn.");
                                continue;
                            }
                        } catch (Exception e) {
                            continue;
                        }
                    } else {
                        dist = (Integer)player.getStats().get("speed");
                    }
                    if(action.charAt(0) == 'w') {
                        int actualPosn = move(player.x(), player.y(), false, -dist, worldMap, mobMap, blocks);
                        player.moveTo(player.x(), actualPosn);
                    } else if (action.charAt(0) == 'a') {
                        int actualPosn = move(player.x(), player.y(), true, -dist, worldMap, mobMap, blocks);
                        player.moveTo(actualPosn, player.y());
                    } else if (action.charAt(0) == 's') {
                        int actualPosn = move(player.x(), player.y(), false, dist, worldMap, mobMap, blocks);
                        player.moveTo(player.x(), actualPosn);
                    } else if (action.charAt(0) == 'd') {
                        int actualPosn = move(player.x(), player.y(), true, dist, worldMap, mobMap, blocks);
                        player.moveTo(actualPosn, player.y());
                    }
                    System.out.println(display((Integer)player.getBaseStats().get("view"),  player.x(), player.y(), worldMap, blocks));

                    if(mobMap[player.x()][player.y()] != 0) {                        
                        mobToFight = new Mob(mobMap[player.x()][player.y()], MOB_FILE);
                        System.out.println("You encountered " + mobToFight.getBaseStats().get("name"));
                        System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("entrance"));
                        System.out.print(mobToFight.getImg());
                        isFightingMob = true;
                    }
                }
            }            
            
        }
        in.close();
    }

    private static StringBuilder map(int[][] worldMap, Block.BlockSet blocks, int chunkSize, Player player) {
        StringBuilder s = new StringBuilder();
        for(int y = 0; y < MAP_SIZE; y += chunkSize) {
            s.append("|");
            for(int x = 0; x < MAP_SIZE; x += chunkSize) {
                if(player.x() >= x && player.x() < x + chunkSize && player.y() > y && player.y() <= y + chunkSize) {
                    s.append(Player.playerRep);
                } else {
                    int majorityBlock = getMajorityBlockInChunk(x, y, chunkSize, blocks, worldMap);
                    int asciiColor = (Integer)blocks.getBlock(majorityBlock).STATS.get("display");
                    if(asciiColor == -1) {
                        s.append("  ");
                    } else {
                        s.append("\033[48;5;" + asciiColor + "m  \033[0m");
                    }
                }
            }
            s.append("|\n");
        }
        return s;
    }

    private static int getMajorityBlockInChunk(int xOrigin, int yOrigin, int chunkSize, Block.BlockSet b, int[][] worldMap) {
        ArrayList<Integer> blocks = new ArrayList<Integer>();
        for(int x = xOrigin; x < xOrigin + chunkSize; x++) {
            for(int y = yOrigin; y < yOrigin + chunkSize; y++) {
                int blockID = worldMap[x][y];
                while(blocks.size() <= blockID) {
                    blocks.add(0);
                }
                blocks.set(blockID, blocks.get(blockID) + 1);
            }
        }
        int maxIndex = 0;
        for(int i = 1; i < blocks.size(); i++) {
            if(blocks.get(i) > blocks.get(maxIndex) || maxIndex == b.getBlock("empty").BLOCK_ID) {
                maxIndex = i;
            }
        }
        return maxIndex;
    }

    private static StringBuilder display(int dist, int xView, int yView, int[][] worldMap, Block.BlockSet blocks) {
        StringBuilder s = new StringBuilder();
        for(int y = max(0,yView - dist); y < min(MAP_SIZE, yView + dist + 1); y++) {
            s.append("|");
            for(int x = max(0,xView - dist); x < min(MAP_SIZE, xView + dist + 1); x++) {                    
                if(x == xView && y == yView) {
                    s.append(Player.playerRep);
                } else {
                    int asciiColor = (Integer)blocks.getBlock(worldMap[x][y]).STATS.get("display");
                    if(asciiColor == -1) {
                        s.append("  ");
                    } else {
                        s.append("\033[48;5;" + asciiColor + "m  \033[0m");
                    }
                }
            }
            s.append("|\n");
        }
        return s;
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

    // calculates the actual move position given the origin, direction to move in, distance to attempt to travel,
    // the mob map, and the world map.
    private static int move(int xOrigin, int yOrigin, boolean xAxis, int numUnits, int[][] worldMap, int[][] mobMap, Block.BlockSet blocks) {
        if(numUnits == 0 && xAxis) {
            return xOrigin;
        } else if (numUnits == 0 && !xAxis) {
            return yOrigin;
        }

        if(xAxis) {
            int bounded = bound(xOrigin + numUnits, 0, MAP_SIZE - 1);
            for(int x = xOrigin + sign(numUnits); x != bounded; x += sign(numUnits)) {
                if((Boolean)blocks.getBlock(worldMap[x][yOrigin]).STATS.get("solid")) {
                    return x - sign(numUnits);
                } else if(mobMap[x][yOrigin] != 0) {
                    Mob m = new Mob(mobMap[x][yOrigin], MOB_FILE);
                    if(RandUtils.rand(0, 99) < (Integer)m.getBaseStats().get("aggression")) {
                        System.out.println("you were blocked by a mob!");
                        return x;
                    }
                }
            }
            boolean solid = (Boolean)blocks.getBlock(worldMap[bounded][yOrigin]).STATS.get("solid");
            return solid ? bounded - sign(numUnits) : bounded;
        } else {
            int bounded = bound(yOrigin + numUnits, 0, MAP_SIZE - 1);
            for(int y = yOrigin + sign(numUnits); y != bounded; y += sign(numUnits)) {
                if((Boolean)blocks.getBlock(worldMap[xOrigin][y]).STATS.get("solid")) {
                    return y - sign(numUnits);
                } else if(mobMap[xOrigin][y] != 0) {
                    Mob m = new Mob(mobMap[xOrigin][y], MOB_FILE);
                    if(RandUtils.rand(0, 99) < (Integer)m.getBaseStats().get("aggression")) {
                        System.out.println("you were blocked by a mob!");
                        return y;
                    }
                }
            }
            boolean solid = (Boolean)blocks.getBlock(worldMap[bounded][yOrigin]).STATS.get("solid");
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
}