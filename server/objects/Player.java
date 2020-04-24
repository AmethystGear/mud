package server.objects;

import java.util.*;

import server.main.World;
import server.utils.RandUtils;
import server.actions.Action;

public class Player {
    public String playerRep = "\033[33m++\033[0m";
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

    private int x;
    private int y;
    private Stats baseStats;
    private Stats stats;
    private Stats inventory;

    public Player(int x, int y, Scanner save) throws Exception {
        moveTo(x, y);
        baseStats = new Stats(save);
        inventory = new Stats(save);
        stats = baseStats.clone();
    }

    public Player(int x, int y) {
        moveTo(x, y);
        baseStats = new Stats();
        baseStats.set("health", DEFAULT_HEALTH);
        baseStats.set("dmg", DEFAULT_DMG);
        baseStats.set("speed", DEFAULT_SPEED);
        baseStats.set("view", DEFAULT_VIEW);
        baseStats.set("xp", DEFAULT_XP);
        stats = baseStats.clone();
        inventory = new Stats();
    }

    public void resetToBaseStats() {
        stats = baseStats.clone();
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

    public Stats.ReadOnlyStats getBaseStats() {
        return new Stats.ReadOnlyStats(baseStats);
    }

    public Stats.ReadOnlyStats getStats() {
        return new Stats.ReadOnlyStats(stats);
    }

    public Mob getMob() {
        return mob;
    }

    public void setMob(Mob m) {
        mob = m;
    }

    public Item getTool() {
        return equippedTool;
    }

    public void setTool(Item i) {
        equippedTool = i;
    }

    public Stats.ReadOnlyStats getInventory() {
        return new Stats.ReadOnlyStats(inventory);
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
        int xp = (Integer) stats.get("xp");
        if (stat.equals("xp")) {
            System.out.println("You can't upgrade your XP!");
        } else if (statLvl * XP_MULTIPLIER <= xp) {
            baseStats.set(stat, statLvl + 1);
            stats.set("xp", xp - statLvl * XP_MULTIPLIER);
            stats.set(stat, baseStats.get(stat));
        } else {
            System.out.println("Not enough XP to upgrade stat. You need " + (statLvl + 1) * XP_MULTIPLIER + " xp.");
        }
        updateXP();
    }

    public void changeStat(String stat, int amount) {
        int currentAmount = (Integer) stats.get(stat);
        stats.set(stat, Math.min(currentAmount + amount, (Integer) baseStats.get(stat)));
        updateXP();
    }

    public void updateXP() {
        baseStats.set("xp", stats.get("xp"));
    }

    public boolean isDead() {
        return (Integer) stats.get("health") <= 0;
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
        return playerRep;
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

        public boolean isDead() {
            return player.isDead();
        }

        public int x() {
            return player.x();
        }

        public int y() {
            return player.y();
        }

        public Mob getMob() {
            return player.getMob();
        }

        public Item getTool() {
            return player.getTool();
        }

        public Stats.ReadOnlyStats getBaseStats() {
            return player.getBaseStats();
        }

        public Stats.ReadOnlyStats getStats() {
            return player.getStats();
        }

        public Stats.ReadOnlyStats getInventory() {
            return player.getInventory();
        }
    }
}
