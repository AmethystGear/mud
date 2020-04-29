package server.objects;

import server.main.World;
import server.utils.RandUtils;
import server.actions.Action;
import server.objects.Stats.ReadOnlyStats;
import server.objects.Position.ReadOnlyPosition;
import server.objects.Int.ReadOnlyInt;

public class Player {
    private int ID;
    private String name;
    public String lastCommand = null;
    public Action lastAction = null;

    private Mob mob;
    private Item equippedTool = null;

    public static final int DEFAULT_HEALTH = 10;
    public static final int DEFAULT_DMG = 1;
    public static final int DEFAULT_SPEED = 5;
    public static final int DEFAULT_XP = 0;
    public static final int DEFAULT_VIEW = 7;
    public static final int XP_MULTIPLIER = 100;

    private Position posn;
    private Stats baseStats;
    private Stats stats;
    private Stats inventory;
    private Int xp;

    public Player(int x, int y, int ID) {
        baseStats = new Stats();
        baseStats.set("health", DEFAULT_HEALTH);
        baseStats.set("dmg", DEFAULT_DMG);
        baseStats.set("speed", DEFAULT_SPEED);
        baseStats.set("view", DEFAULT_VIEW);
        stats = baseStats.clone();
        inventory = new Stats();
        name = null;
        posn = new Position(x, y);
        xp = new Int(DEFAULT_XP);
        this.ID = ID;
    }

    public void login(String name, ReadOnlyStats inventory, ReadOnlyStats stats, ReadOnlyPosition posn, ReadOnlyInt xp) {
        this.name = name;
        this.inventory = inventory.clone();
        this.baseStats = stats.clone();
        this.stats = stats.clone();
        this.posn = new Position(posn.x(), posn.y());
        this.xp = new Int(xp.get());
    }

    public int ID() {
        return ID;
    }

    public ReadOnlyInt xp() {
        return new ReadOnlyInt(xp);
    }

    public String getName() {
        return name;
    }

    public int x() {
        return posn.x();
    }

    public int y() {
        return posn.y();
    }

    public Position.ReadOnlyPosition getPosn() {
        return new Position.ReadOnlyPosition(posn);
    }

    public Stats.ReadOnlyStats getBaseStats() {
        return new Stats.ReadOnlyStats(baseStats);
    }

    public Stats.ReadOnlyStats getStats() {
        return new Stats.ReadOnlyStats(stats);
    }

    public Mob getMob() {
        return mob;
    }

    public Item getTool() {
        return equippedTool;
    }

    public Stats.ReadOnlyStats getInventory() {
        return new Stats.ReadOnlyStats(inventory);
    }

    public boolean isDead() {
        return (Integer) stats.get("health") <= 0;
    }

    public void resetToBaseStats() {
        stats = baseStats.clone();
    }

    public void moveTo(int x, int y) {
        posn.moveTo(x, y);
    }

    public void setMob(Mob m) {
        mob = m;
    }

    public void setTool(Item i) {
        equippedTool = i;
    }

    public void removeFromInventory(String item, int count) {
        if (count < 0 || !inventory.hasVariable(item)) {
            throw new IllegalArgumentException();
        }
        int numItemInInventory = (Integer) inventory.get(item);
        if (numItemInInventory - count < 0) {
            throw new IllegalArgumentException();
        }
        if (numItemInInventory - count == 0) {
            inventory.removeVariable(item);
        } else {
            inventory.set(item, numItemInInventory - count);
        }
    }

    public void upgradeBaseStat(String stat) {
        int statLvl = (Integer) baseStats.get(stat);
        baseStats.set(stat, statLvl + 1);
        xp.set(xp.get() - statLvl * XP_MULTIPLIER);
        stats.set(stat, baseStats.get(stat));
    }

    public void changeXP(int delta) {
        xp.set(xp.get() + delta);
    }

    public void changeStat(String stat, int amount) {
        int currentAmount = (Integer) stats.get(stat);
        stats.set(stat, Math.min(currentAmount + amount, (Integer) baseStats.get(stat)));        
    }

    public void addToInventory(String item, int count) {
        if (count < 0) {
            throw new IllegalArgumentException();
        }
        if (inventory.hasVariable(item)) {
            int amount = (Integer) inventory.get(item);
            inventory.set(item, amount + count);
        } else {
            inventory.set(item, count);
        }
    }

    public void clearInventory() {
        inventory = new Stats();
    }

    public String toString() {
        return ID + " " + ID;
    }

    public void respawn(World world) {
        clearInventory();
        resetToBaseStats();
        mob = null;

        int spawnX = RandUtils.rand(0, World.MAP_SIZE - 1);
        int spawnY = RandUtils.rand(0, World.MAP_SIZE - 1);
        Block b = world.getBlock(spawnX, spawnY);
        while (b.getStats().hasProperty("solid") || ((String) b.getStats().get("name")).contains("water")
                || world.hasMob(spawnX, spawnY)) {
            spawnX = RandUtils.rand(0, World.MAP_SIZE - 1);
            spawnY = RandUtils.rand(0, World.MAP_SIZE - 1);
            b = world.getBlock(spawnX, spawnY);
        }
        moveTo(spawnX, spawnY);
    }

    public static class ReadOnlyPlayer {
        private Player player;

        public ReadOnlyPlayer(Player player) {
            this.player = player;
        }

        public ReadOnlyInt xp() {
            return player.xp();
        }
    
        public String getName() {
            return player.getName();
        }
    
        public int x() {
            return player.x();
        }
    
        public int y() {
            return player.y();
        }
    
        public Position.ReadOnlyPosition getPosn() {
            return player.getPosn();
        }
    
        public Stats.ReadOnlyStats getBaseStats() {
            return player.getBaseStats();
        }
    
        public Stats.ReadOnlyStats getStats() {
            return player.getStats();
        }
    
        public Mob getMob() {
            return player.getMob();
        }
    
        public Item getTool() {
            return player.getTool();
        }
    
        public Stats.ReadOnlyStats getInventory() {
            return player.getInventory();
        }
    
        public boolean isDead() {
            return player.isDead();
        }

        public int ID() {
            return player.ID();
        }
    }
}
