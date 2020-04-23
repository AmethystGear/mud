package server.objects;

import server.utils.RandUtils;
import server.main.World;

public class Mob implements ValueType<Mob> {
    private int ID;
    private Stats stats;
    private Stats baseStats;

    public Mob() {
    }

    public Mob(int ID, Stats.ReadOnlyStats stats) {
        this.ID = ID;
        this.stats = stats.clone();
        this.baseStats = stats.clone();
    }

    @Override
    public int getID() {
        return ID;
    }

    @Override
    public Stats.ReadOnlyStats getStats() {
        return new Stats.ReadOnlyStats(stats);
    }

    @Override
    public Mob create(int ID, Stats.ReadOnlyStats stats) {
        return new Mob(ID, stats);
    }

    public Stats.ReadOnlyStats getBaseStats() {
        return new Stats.ReadOnlyStats(baseStats);
    }

    public boolean isDead() {
        return (Integer) stats.get("health") <= 0;
    }

    public String[] getDrops() {
        if (!stats.hasVariable("drops")) {
            return new String[0];
        }
        String[] drops = (String[]) baseStats.get("drops");
        int min = stats.hasVariable("drop min") ? (Integer) getBaseStats().get("drop min") : 1;
        int max = stats.hasVariable("drop min") ? (Integer) getBaseStats().get("drop min") : 1;
        int numDrops = RandUtils.rand(min, max);
        String[] playerDrops = new String[numDrops];
        for (int i = 0; i < playerDrops.length; i++) {
            playerDrops[i] = RandUtils.getRandom(drops);
        }
        return playerDrops;
    }

    public String getQuote(String quoteType) {
        if (!stats.hasVariable(quoteType)) {
            return "";
        }
        return RandUtils.getRandom((String[]) stats.get(quoteType));
    }

    public String getImg() {
        return baseStats.hasVariable("img") ? ((StringBuilder) baseStats.get("img")).toString() : "\n";
    }
    
    public void changeStat(String stat, int amount) {
        int currentAmount = (Integer) stats.get(stat);
        stats.set(stat, currentAmount + amount);
    }

    public StringBuilder attack(Player player, World world) {
        StringBuilder out = new StringBuilder("");
        int playerSpeed = (Integer)player.getStats().get("speed");
        int mobSpeed = (Integer)stats.get("speed");
        int numTurns = mobSpeed / playerSpeed == 0 ? 1 : mobSpeed / playerSpeed;
        for(int i = 0; i < numTurns; i++) {
            out.append(getBaseStats().get("name") + ": " + getQuote("attack") + "\n");
            out.append(getBaseStats().get("name") + " attacked you and dealt " + getBaseStats().get("dmg") + " damage.\n");
            player.changeStat("health", -(Integer)getStats().get("dmg"));
            if(player.isDead()) {
                out.append(getBaseStats().get("name") + ": " + getQuote("mob-victory") + "\n");
                out.append("You were killed by " + getBaseStats().get("name") + "\n");
                player.respawn(world);
                out.append("Respawning at " + player.x() + ", " + player.y() + "\n");
                return out;
            }
            if(i < numTurns - 1) {
                out.append(stats.get("name") + " is faster than you, and takes another turn.\n");
            }
        }
        return out;
    }

    public static class ReadOnlyMob implements ValueType<ReadOnlyMob> {
        private Mob m;
    
        public ReadOnlyMob(Mob m) {
            this.m = m;
        }
    
        @Override
        public int getID() {
            return m.getID();
        }
    
        @Override
        public Stats.ReadOnlyStats getStats() {
            return m.getStats();
        }
    
        @Override
        public ReadOnlyMob create(int ID, Stats.ReadOnlyStats stats) {
            return new ReadOnlyMob(m.create(ID, stats));
        }
    
        public Mob clone() {
            return m.create(m.getID(), m.getBaseStats());
        }
        
        public Stats.ReadOnlyStats getBaseStats() {
            return m.getBaseStats();
        }
    
        public boolean isDead() {
            return m.isDead();
        }
    
        public String[] getDrops() {
            return m.getDrops();
        }
    
        public String getQuote(String quoteType) {
            return m.getQuote(quoteType);
        }
    
        public String getImg() {
            return m.getImg();
        }
    }
}