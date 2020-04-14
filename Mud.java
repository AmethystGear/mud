import java.io.*;
import java.util.*;
import java.util.concurrent.ThreadLocalRandom;

public class Mud {
    private static final int MAP_SIZE = 3000;
    private static final int MOB_SPAWN_CHANCE = 10;
    private static final int MOB_SPAWN_CHANCE_GRASS = 40;
    private static final int GRASS_BLOCK_SPAWN_CHANCE = 10;
    private static final int GRASS_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR = 65;
    private static final int FILLED_BLOCK_SPAWN_CHANCE = 5;
    private static final int FILLED_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR = 50;

    private static final String[] BLOCK_TYPES = new String[]{"  ", 
                                                             "\033[48;5;149m  \033[0m",
                                                             "\033[48;5;245m  \033[0m",
                                                             "\033[48;5;136m  \033[0m",
                                                             "\033[48;5;94m  \033[0m"};
    private static final String[] BLOCK_TYPES_DESCR = new String[]{"plain", "grass", "rock", "village floor", "village wall"};
    private static final String MOB_FILE = "mobs.txt";
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


    private static void spawnVillage(int xOrigin, int yOrigin, int [][] worldMap, int[][] mobMap) {
        int villageLength = rand(8, 30) * 5;
        int pathSize = rand(3, 5);
        for(int x = xOrigin; x < xOrigin + villageLength; x++) {
            for(int y = yOrigin; y < yOrigin + pathSize; y++) {
                worldMap[x][y] = 3;
                mobMap[x][y] = 0;
            }
        }
        boolean generateUp = false;
        for(int x = xOrigin + rand(2, 5); x < xOrigin + villageLength; x+= rand(5, 10)) {
            int pathlen = rand(3, 10);
            int hutSize = rand(2, 4);
            if(generateUp) {
                for(int y = yOrigin; y > yOrigin - pathlen; y--) {
                    worldMap[x][y] = 3;
                    mobMap[x][y] = 0;
                }
                spawnHut(x - hutSize, yOrigin - pathlen - hutSize * 2 + 1, hutSize, worldMap, mobMap);
            } else {
                for(int y = yOrigin + pathSize; y < yOrigin + pathlen + pathSize; y++) {
                    worldMap[x][y] = 3;
                    mobMap[x][y] = 0;
                }
                spawnHut(x - hutSize, yOrigin + pathlen + pathSize, hutSize, worldMap, mobMap);
            }
            generateUp = !generateUp;
        }
    }

    private static void spawnHut(int xOrigin, int yOrigin, int size, int [][] worldMap, int[][] mobMap) {
        size = size * 2 + 1;
        for(int x = xOrigin; x < xOrigin + size; x++) {
            for(int y = yOrigin; y < yOrigin + size; y++) {
                worldMap[x][y] = 3;
                mobMap[x][y] = 0;
            }
        }
        for(int x = xOrigin; x < xOrigin + size; x++) {
            if(x - xOrigin != size/2) {
                worldMap[x][yOrigin] = 4;
                worldMap[x][yOrigin + size - 1] = 4;
            }
        }
        for(int y = yOrigin; y < yOrigin + size; y++) {
            worldMap[xOrigin][y] = 4;
            worldMap[xOrigin + size - 1][y] = 4;
        }
    }

    private static String[] getRemainingInputAsStringArray(Scanner s) {
        ArrayList<String> a = new ArrayList<>();
        while(s.hasNext()) {
            String next = s.next();
            String spacesAdded = next.replace('-', ' ').replace('_', ' ');
            a.add(spacesAdded);

        }
        String [] arr = new String[a.size()];
        for(int i = 0; i < a.size(); i++) {
            arr[i] = a.get(i);
        }
        return arr;
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
                    String[] drops = getRemainingInputAsStringArray(tok);
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
                int blockType = 0;
                if(rand(0, 99) < GRASS_BLOCK_SPAWN_CHANCE) {
                    blockType = 1;
                }
                if(hasNeighbor(x, y, 1, worldMap) && rand(0, 99) < GRASS_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR) {
                    blockType = 1;
                }
                if(rand(0, 99) < FILLED_BLOCK_SPAWN_CHANCE) {
                    blockType = 2;
                }
                if(hasNeighbor(x, y, 2, worldMap) && rand(0, 99) < FILLED_BLOCK_SPAWN_CHANCE_IF_NEIGHBOR) {
                    blockType = 2;
                }
                worldMap[x][y] = blockType;

                if(blockType == 0) {
                    if(rand(0, 99) < MOB_SPAWN_CHANCE) {
                        mobMap[x][y] = rand(1, NUM_MOB_TYPES);
                    }
                } else if(blockType == 1) {
                    if(rand(0, 99) < MOB_SPAWN_CHANCE_GRASS) {
                        mobMap[x][y] = rand(1, NUM_MOB_TYPES);
                    }
                }
            }
        }

        int numVillages = rand(50, 100);
        for(int i = 0; i < numVillages; i++) {
            int x = rand(500, 2500);
            int y = rand(500, 2500);
            System.out.println(x + ", " + y);
            spawnVillage(x, y, worldMap, mobMap);
        }
        
        // assign spawn location to a place that is open and doesn't have a mob.
        int spawnX = rand(0, MAP_SIZE - 1);
        int spawnY = rand(0, MAP_SIZE - 1);
        while(worldMap[spawnX][spawnY] == 2|| mobMap[spawnX][spawnY] != 0) {
            spawnX = rand(0, MAP_SIZE - 1);
            spawnY = rand(0, MAP_SIZE - 1);
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
                    System.out.println("You attacked " + mobToFight.name() + " and dealt " + player.getBaseStats().get("dmg") + " damage.");
                    mobToFight.changeStat("health", -player.getBaseStats().get("dmg"));
                    if(mobToFight.isDead()) {
                        System.out.println(mobToFight.name() + ": " + mobToFight.getQuote("player-victory"));
                        System.out.println("You murdered " + mobToFight.name());
                        mobMap[player.x()][player.y()] = 0; // remove mob from map
                        System.out.println("You got " + mobToFight.getBaseStats().get("xp") + " xp.");
                        player.changeStat("xp", mobToFight.getBaseStats().get("xp"));

                        String[] drops = mobToFight.getDrops();
                        for(String drop : drops) {
                            System.out.println("You got " + drop);
                            player.addToInventory(drop);
                        }

                        isFightingMob = false;
                    } else {
                        System.out.println(mobToFight.name() + ": " + mobToFight.getQuote("attack"));
                        System.out.println(mobToFight.name() + " attacked you and dealt " + mobToFight.getBaseStats().get("dmg") + " damage.");
                        player.changeStat("health", -mobToFight.getBaseStats().get("dmg"));
                        if(player.isDead()) {
                            System.out.println(mobToFight.name() + ": " + mobToFight.getQuote("mob-victory"));
                            System.out.println("You were killed by " + mobToFight.name());
                            return;
                        }
                    }
                } else if(action.equals("run")) {
                    System.out.println(mobToFight.name() + ": " + mobToFight.getQuote("player-run"));
                    System.out.println("You ran away from " + mobToFight.name() + ".");
                    isFightingMob = false;
                } else if(action.equals("trade")) {
                    int numItems;
                    int xp;
                    try {
                        numItems = mobToFight.getBaseStats().get("trade");
                        xp = mobToFight.getBaseStats().get("trade-xp");
                    } catch(IllegalArgumentException e) {
                        System.out.println("You can't trade with " + mobToFight + "!");
                        numItems = -1;
                        xp = -1;
                    }
                    if(numItems != -1) {                      
                        int x = player.x();
                        int y = player.y();
                        //get hash that will be unique for all x, y
                        //this only works when x, y << INT_MAX, so make sure the map
                        //isn't too big.
                        int hash = x * (int)Math.pow(10, (x + "").length()) + y; 
                        Random XYRand = new Random(hash);

                        System.out.println("I can trade " + xp + " xp for each: ");
                        String[] trades = new String[numItems];
                        for(int i = 0; i < trades.length; i++) {
                            trades[i] = allDrops[XYRand.nextInt(allDrops.length)];
                            System.out.println((i + 1) + ". " + trades[i]);
                        }
                        System.out.print("Enter which # item you wish to trade: ");
                        int itemNum = Integer.parseInt(in.nextLine()) - 1;
                        try {
                            int amount = player.getInventory().get(trades[itemNum]);
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
                    System.out.print("Enter how far: ");
                    int dist = Integer.parseInt(in.nextLine());
                    System.out.println(display(dist, player, worldMap));
                } else if(action.equals("w") || action.equals("a") || action.equals("s") || action.equals("d")) { // movement
                    System.out.print("Enter how far: ");
                    int dist = Integer.parseInt(in.nextLine());
                    if(action.equals("w")) {
                        int actualPosn = move(player.x(), player.y(), false, -dist, worldMap, mobMap);
                        player.moveTo(player.x(), actualPosn);
                    } else if (action.equals("a")) {
                        int actualPosn = move(player.x(), player.y(), true, -dist, worldMap, mobMap);
                        player.moveTo(actualPosn, player.y());
                    } else if (action.equals("s")) {
                        int actualPosn = move(player.x(), player.y(), false, dist, worldMap, mobMap);
                        player.moveTo(player.x(), actualPosn);
                    } else if (action.equals("d")) {
                        int actualPosn = move(player.x(), player.y(), true, dist, worldMap, mobMap);
                        player.moveTo(actualPosn, player.y());
                    }
                    System.out.println(display(10, player, worldMap));

                    if(mobMap[player.x()][player.y()] != 0) {                        
                        mobToFight = new Mob(mobMap[player.x()][player.y()], MOB_FILE);
                        System.out.println("You encountered " + mobToFight.name());
                        System.out.println(mobToFight.name() + ": " + mobToFight.getQuote("entrance"));
                        System.out.print(mobToFight.getImg());
                        isFightingMob = true;
                    }
                }
            }            
            
        }
        in.close();
    }

    private static StringBuilder display(int dist, Player player, int[][] worldMap) {
        System.out.println("You are at position: " + player.x() + ", " + player.y());
        StringBuilder s = new StringBuilder();
        for(int y = max(0,player.y() - dist); y < min(MAP_SIZE, player.y() + dist + 1); y++) {
            s.append("|");
            for(int x = max(0,player.x() - dist); x < min(MAP_SIZE, player.x() + dist + 1); x++) {                    
                if(x == player.x() && y == player.y()) {
                    s.append(player.toString());
                } else {
                    s.append(BLOCK_TYPES[worldMap[x][y]]);
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

    private static int rand(int minInc, int maxInc) {
        return ThreadLocalRandom.current().nextInt(minInc, maxInc + 1);
    }

    // calculates the actual move position given the origin, direction to move in, distance to attempt to travel,
    // the mob map, and the world map.
    private static int move(int xOrigin, int yOrigin, boolean xAxis, int numUnits, int[][] worldMap, int[][] mobMap) {
        if(numUnits == 0 && xAxis) {
            return xOrigin;
        } else if (numUnits == 0 && !xAxis) {
            return yOrigin;
        }

        if(xAxis) {      
            int bounded = bound(xOrigin + numUnits, 0, MAP_SIZE - 1);
            for(int x = xOrigin + sign(numUnits); x != bounded; x += sign(numUnits)) {
                if(worldMap[x][yOrigin] == 2) {
                    return x - sign(numUnits);
                } else if(mobMap[x][yOrigin] != 0) {
                    Mob m = new Mob(mobMap[x][yOrigin], MOB_FILE);
                    if(rand(0, 99) < m.getBaseStats().get("aggression")) {
                        System.out.println("you were blocked by a mob!");
                        return x;
                    }
                }
            }
            return worldMap[bounded][yOrigin] == 2 ? bounded - sign(numUnits) : bounded;
        } else {
            int bounded = bound(yOrigin + numUnits, 0, MAP_SIZE - 1);
            for(int y = yOrigin + sign(numUnits); y != bounded; y += sign(numUnits)) {
                if(worldMap[xOrigin][y] == 2) {
                    return y - sign(numUnits);
                } else if(mobMap[xOrigin][y] != 0) {
                    Mob m = new Mob(mobMap[xOrigin][y], MOB_FILE);
                    if(rand(0, 99) < m.getBaseStats().get("aggression")) {
                        System.out.println("you were blocked by a mob!");
                        return y;
                    }
                }
            }
            return worldMap[xOrigin][bounded] == 2 ? bounded - sign(numUnits) : bounded;
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

class Stats {
    private HashMap<String, Integer> stats;
    public Stats () {
        stats = new HashMap<>();
    }

    public Stats (String saveFile) throws FileNotFoundException {
        stats = new HashMap<>();
        Scanner scan = new Scanner(new File(saveFile));
        while(scan.hasNext()) {
            String s = scan.next().replace('-', ' ');
            int i = scan.nextInt();
            set(s, i);
        }
    }

    // get a stat. If it doesn't exist, throw an IllegalArgumentException.
    public int get(String stat) {
        if(!stats.containsKey(stat) || stats.get(stat) == null) {
            throw new IllegalArgumentException();
        }
        return stats.get(stat);
    }

    // set a stat to a value. If the stat doesn't exist, create it.
    public void set(String stat, int value) {
        stats.put(stat, value);
    }

    // change an already-existing stat by the value.
    // throws IllegalArgumentException if the stat doesn't exist.
    public void change(String stat, int value) {
        if(!stats.containsKey(stat)) {
            throw new IllegalArgumentException();
        }
        stats.put(stat, stats.get(stat) + value);
    }

    // creates a deep copy of stats
    public Stats clone() {
        Stats newStats = new Stats();
        for(Map.Entry<String, Integer> e : stats.entrySet()) {
            newStats.set(e.getKey(), e.getValue());
        }
        return newStats;
    }

    public void saveTo(String file) throws Exception {
        PrintWriter writer = new PrintWriter(file, "UTF-8");
        for(Map.Entry<String, Integer> e : stats.entrySet()) {
            writer.write(e.getKey().replace(' ', '-'));
            writer.write(" ");
            writer.write(e.getValue() + "");
            writer.write("\n");
        }
        writer.close();
    }

    // string rep of the stats.
    public String toString() {
        StringBuilder s = new StringBuilder();
        for(Map.Entry<String, Integer> e : stats.entrySet()) {
            s.append(e.getKey());
            s.append(": ");
            s.append(e.getValue());
            s.append("\n");
        }
        return s.toString();
    }
}

class ReadOnlyStats {
    private Stats stats;
    public ReadOnlyStats(Stats stats) {
        this.stats = stats;
    }
    public int get(String stat) {
        return stats.get(stat);
    }
    public String toString() {
        return stats.toString();
    }
    public void saveTo(String file) throws Exception {
        stats.saveTo(file);
    }
}

class Player {
    public static String playerRep = "\033[33m++\033[0m";
    public static final int DEFAULT_HEALTH = 10;
    public static final int DEFAULT_DMG = 1;
    public static final int DEFAULT_SPEED = 5;
    public static final int DEFAULT_XP = 0;
    public static final int XP_MULTIPLIER = 100;

    private int x;
    private int y;
    private Stats baseStats;
    private Stats stats;
    private Stats inventory;

    public Player(int x, int y, String statsSave, String inventorySave) throws Exception {
        moveTo(x, y);
        baseStats = new Stats(statsSave);
        stats = baseStats.clone();
        inventory = new Stats(inventorySave);
    }

    public Player(int x, int y) {
        moveTo(x, y);
        baseStats = new Stats();
        baseStats.set("health", DEFAULT_HEALTH);
        baseStats.set("dmg", DEFAULT_DMG);
        baseStats.set("speed", DEFAULT_SPEED);
        baseStats.set("xp", DEFAULT_XP);
        stats = baseStats.clone();
        inventory = new Stats();
    }

    public void moveTo(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public int x() {
        return x;
    }

    public int y() {
        return y;
    }

    public ReadOnlyStats getBaseStats() {
        return new ReadOnlyStats(baseStats);
    }

    public ReadOnlyStats getStats() {
        return new ReadOnlyStats(stats);
    }

    public ReadOnlyStats getInventory() {
        return new ReadOnlyStats(inventory);
    }

    public void removeFromInventory(String item, int count) {
        if(inventory.get(item) - count < 0) {
            throw new IllegalArgumentException();
        }
        inventory.change(item, -count);
    }

    public void upgradeBaseStat(String stat) {
        if(stat.equals("xp")) {
            System.out.println("You can't upgrade your XP!");
        } else if ((baseStats.get(stat) + 1) * XP_MULTIPLIER <= stats.get("xp")) {
            baseStats.change(stat, 1);
            stats.change("xp", -baseStats.get(stat) * XP_MULTIPLIER);
            stats.set(stat, baseStats.get(stat));
        } else {
            System.out.println("Not enough XP to upgrade stat. You need " + (baseStats.get(stat) + 1) * XP_MULTIPLIER + " xp.");
        }
        updateXP();
    }

    public void changeStat(String stat, int amount) {
        stats.change(stat, amount);
        updateXP();
    }

    public void updateXP() {
        baseStats.set("xp", stats.get("xp"));
    }

    public boolean isDead() {
        return stats.get("health") <= 0;
    }

    public void addToInventory(String item) {
        try {
            inventory.change(item, 1);
        } catch(IllegalArgumentException e) {
            inventory.set(item, 1);
        }        
    }

    public String toString() {
        return playerRep;
    }        
}

class Mob {
    private String name;
    private StringBuilder img = new StringBuilder("");
    private HashMap<String, String[]> quoteTypeToQuoteList;
    private String[] drops;
    private Stats baseStats;
    private Stats stats;


    public Mob(int mobType, String mobFile) {
        baseStats = new Stats();
        stats = new Stats();
        try {
            quoteTypeToQuoteList = new HashMap<>();
            int mob = 0;
            boolean gettingImg = false;
            Scanner fr = new Scanner(new File(mobFile));
            while (fr.hasNextLine()) {
                String data = fr.nextLine();
                Scanner tokenizer = new Scanner(data);
                if(!tokenizer.hasNext()) { //ignore empty lines
                    continue;
                }
                if(data.strip().equals("/begin/")) {
                    mob++;
                }
                else if(mob == mobType) {                    
                    String dataType = tokenizer.next();
                    if(gettingImg) {
                        img.append(data);
                        img.append("\n");
                    } else if(dataType.equals("name:")) {
                        this.name = getRemainingInputAsString(tokenizer);
                    } else if(dataType.equals("img:")) {
                        gettingImg = true;
                    } else if(dataType.equals("drops:")) {
                        drops = getRemainingInputAsStringArray(tokenizer);
                    } else if(tokenizer.hasNextInt()) {
                        String colonRemoved = dataType.substring(0, dataType.length() - 1);
                        baseStats.set(colonRemoved, tokenizer.nextInt());
                    } else {
                        String[] quotes = getRemainingInputAsStringArray(tokenizer);
                        String colonRemoved = dataType.substring(0, dataType.length() - 1);
                        quoteTypeToQuoteList.put(colonRemoved, quotes);
                    }
                }
            }
            fr.close();
        } catch (FileNotFoundException e) {
            System.out.println("An error occurred.");
            e.printStackTrace();
        }
        stats = baseStats.clone();
    }

    public ReadOnlyStats getBaseStats() {
        return new ReadOnlyStats(stats);
    }

    public ReadOnlyStats getStats() {
        return new ReadOnlyStats(stats);
    }

    public void changeStat(String stat, int amount) {
        stats.change(stat, amount);
    }

    public boolean isDead() {
        return stats.get("health") <= 0;
    }

    public String[] getDrops() {
        if(drops == null) {
            return new String[0];
        }
        int min;
        int max;
        try {
            min = getBaseStats().get("drop-min");
        } catch (IllegalArgumentException e) {
            min = 1;
        }
        try {
            max = getBaseStats().get("drop-max");
        } catch (IllegalArgumentException e) {
            max = min;
        }
        int numDrops = rand(min, max);
        String[] playerDrops = new String[numDrops];
        for(int i = 0; i < playerDrops.length; i++) {
            playerDrops[i] = getRandom(drops);
        }
        return playerDrops;
    }

    public String name() {
        return name;
    }

    public String getQuote(String quoteType) {
        if(!quoteTypeToQuoteList.keySet().contains(quoteType) ||
            quoteTypeToQuoteList.get(quoteType) == null ||
            quoteTypeToQuoteList.get(quoteType).length == 0) {
            return "";
        }
        return getRandom(quoteTypeToQuoteList.get(quoteType));
    }

    public String getImg() {
        return img.toString();
    }

    private static int rand(int minInc, int maxInc) {
        return ThreadLocalRandom.current().nextInt(minInc, maxInc + 1);
    }

    private static String getRandom(String[] array) {
        return array[rand(0, array.length - 1)];
    }

    private static String getRemainingInputAsString(Scanner s) {
        StringBuilder str = new StringBuilder();
        while(s.hasNext()) {
            str.append(s.next());
            if(s.hasNext()) {
                str.append(" ");
            }            
        }
        return str.toString();
    }

    private static String[] getRemainingInputAsStringArray(Scanner s) {
        ArrayList<String> a = new ArrayList<>();
        while(s.hasNext()) {
            String next = s.next();
            String spacesAdded = next.replace('-', ' ').replace('_', ' ');
            a.add(spacesAdded);

        }
        String [] arr = new String[a.size()];
        for(int i = 0; i < a.size(); i++) {
            arr[i] = a.get(i);
        }
        return arr;
    }
}