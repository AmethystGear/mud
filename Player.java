import java.io.*;
import java.util.*;

public class Player {
    public String playerRep = "\033[33m++\033[0m";
    public String lastCommand = "";
    public Mob mob;

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

    public Player(int x, int y, String statsSave, String inventorySave) throws Exception {
        moveTo(x, y);
        baseStats = new Stats(new Scanner(new File(statsSave)));
        inventory = new Stats(new Scanner(new File(inventorySave)));
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
        if(count < 0 || !inventory.hasVariable(item)) {
            throw new IllegalArgumentException();
        }
        int numItemInInventory = (Integer)inventory.get(item);
        if(numItemInInventory - count < 0) {
            throw new IllegalArgumentException();
        }
        if(numItemInInventory - count == 0) {
            inventory.removeVariable(item);
        } else {
            inventory.set(item, numItemInInventory - count);
        }
    }

    public void upgradeBaseStat(String stat) {
        int statLvl = (Integer)baseStats.get(stat);
        int xp = (Integer)stats.get("xp");
        if(stat.equals("xp")) {
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
        int currentAmount = (Integer)stats.get(stat);
        stats.set(stat, Math.min(currentAmount + amount, (Integer)baseStats.get("health")));
        updateXP();
    }

    public void updateXP() {
        baseStats.set("xp", stats.get("xp"));
    }

    public boolean isDead() {
        return (Integer)stats.get("health") <= 0;
    }

    public void addToInventory(String item, int count) {
        if(count < 0 || !inventory.hasVariable(item)) {
            throw new IllegalArgumentException();
        }
        if(inventory.hasVariable(item)) {
            int amount = (Integer)inventory.get(item);
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
}
