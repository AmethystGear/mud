import java.io.*;
import java.util.*;

public class Mud {
    

    //files that you can use to configure the game
    private static final String MOB_FILE = "mobs.txt";
    private static final String BLOCKS_FILE = "blocks.txt";

    // player save files
    private static final String STATS_SAVE = "stats-save.txt";
    private static final String INVENTORY_SAVE = "inventory-save.txt";

    // world save files
    private static final String WORLD_SAVE = "world-save.txt";

    private static int NUM_MOB_TYPES = 0;

    private static boolean fileExists(String file) {
        return new File(file).exists() && !new File(file).isDirectory();
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

        Block.BlockSet blocks = Block.getBlocksFromFile(BLOCKS_FILE);

        boolean makeNewWorld;
        Scanner in = new Scanner(System.in);
        System.out.print("Do you want to load your saved world, or create a new one?(load/create): ");
        String inp = in.nextLine();
        while(!inp.equals("load") && !inp.equals("create")) {
            System.out.print("Please type load or create: ");
            inp = in.nextLine();
        }
        makeNewWorld = inp.equals("create");
        
        World world;
        if (makeNewWorld) {
            world = new World(RandUtils.rand(0, Integer.MAX_VALUE - 1), NUM_MOB_TYPES, blocks);
        } else {
            world = new World(WORLD_SAVE, NUM_MOB_TYPES, blocks);
        }
        
        // assign spawn location to a place that is open and doesn't have a mob.
        int spawnX = RandUtils.rand(0, MAP_SIZE - 1);
        int spawnY = RandUtils.rand(0, MAP_SIZE - 1);
        Block b = world.getBlock(spawnX, spawnY);
        while((Boolean)b.STATS.get("solid") || ((String)b.STATS.get("name")).contains("water")) {
            spawnX = RandUtils.rand(0, MAP_SIZE - 1);
            spawnY = RandUtils.rand(0, MAP_SIZE - 1);
            b = world.getBlock(spawnX, spawnY);
        }

        Player player;
        if(fileExists(STATS_SAVE) && fileExists(INVENTORY_SAVE)) {
            player = new Player(spawnX, spawnY, STATS_SAVE, INVENTORY_SAVE);
        } else {
            player = new Player(spawnX, spawnY);
        }

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
                saveMap(MOB_SAVE, mobMap);
                saveMap(WORLD_SAVE, worldMap);
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
                            System.out.println((i + 1) + ". " + trades[i].replace(' ', '-'));
                        }
                        System.out.print("Enter which # item you wish to trade: ");
                        int itemNum = Integer.parseInt(in.nextLine()) - 1;
                        try {
                            int amount = (Integer)player.getInventory().get(trades[itemNum].replace(' ', '-'));
                            System.out.print("You have " + amount + " of that item. How many do you wish to trade? ");
                            int numToTrade = Integer.parseInt(in.nextLine());
                            try {
                                player.removeFromInventory(trades[itemNum].replace(' ', '-'), numToTrade);
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
                        System.out.println(display(dist, xPos, yPos, worldMap, mobMap, blocks));
                    } else {
                        System.out.println(display((Integer)player.getBaseStats().get("view"), player.x(), player.y(), worldMap, mobMap, blocks));
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
                    System.out.println(display((Integer)player.getBaseStats().get("view"),  player.x(), player.y(), worldMap, mobMap, blocks));

                    if(mobMap[player.x()][player.y()] != 0) {                        
                        mobToFight = new Mob(mobMap[player.x()][player.y()], MOB_FILE);
                        System.out.println("You encountered " + mobToFight.getBaseStats().get("name"));
                        System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("entrance"));
                        System.out.print(mobToFight.getImg());
                        if((Integer)mobToFight.getStats().get("speed") > (Integer)player.getStats().get("speed")) {
                            System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("attack"));
                            System.out.println(mobToFight.getBaseStats().get("name") + " attacked you and dealt " + mobToFight.getBaseStats().get("dmg") + " damage.");
                            player.changeStat("health", -(Integer)mobToFight.getBaseStats().get("dmg"));
                            if(player.isDead()) {
                                System.out.println(mobToFight.getBaseStats().get("name") + ": " + mobToFight.getQuote("mob-victory"));
                                System.out.println("You were killed by " + mobToFight.getBaseStats().get("name"));
                                return;
                            }
                        }
                        isFightingMob = true;
                    }
                }
            }            
            
        }
        in.close();
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